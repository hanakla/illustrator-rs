use std::ffi::{c_char, c_void, CStr};
use std::ptr::null_mut;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ai_plugin::AIPlugin;
use illustrator_sys::ai_sys::SPPluginRef;


/// Suites 構造体
#[repr(C)]
pub struct Suites {
    // 実際のフィールドは省略
}

impl Suites {
    pub fn new() -> Result<Box<Self>, ASErr> {
        Ok(Box::new(Self {}))
    }

    pub fn Error(&self) -> ASErr {
        kNoErr
    }

    pub fn InitializeRefCount(&mut self) {
    }

    pub fn acquire_Optional_Suites(&mut self) {
    }
}


#[repr(C)]
pub struct Plugin<T: AIPlugin> {
    pub fPluginRef: SPPluginRef,
    pub fSuites: *mut Suites,
    pub fPluginName: [i8; kMaxStringLength],
    pub fLockCount: i32,
    pub fPluginAccess: *mut c_void,
    pub fLastError: ASErr,
    pub fSupressDuplicateErrors: bool,
    pub fErrorTimeout: i64,
    pub fLastErrorTime: i64,
    pub fApplicationStartedNotifier: *mut c_void,
    pub fApplicationShutdownNotifer: *mut c_void,
    pub handler: T,
}

/// プラグイン実装の関数群
impl<T: AIPlugin + Default> Plugin<T> {
    /// 新しいプラグインインスタンスを作成
    pub fn new(plugin_ref: SPPluginRef, plugin_name: &str) -> Self {
        let mut plugin = Self {
            fPluginRef: plugin_ref,
            fSuites: null_mut(),
            fPluginName: [0; kMaxStringLength],
            fLockCount: 0,
            fPluginAccess: null_mut(),
            fLastError: kNoErr,
            fSupressDuplicateErrors: true,
            fErrorTimeout: 5, // seconds
            fLastErrorTime: 0,
            fApplicationStartedNotifier: null_mut(),
            fApplicationShutdownNotifer: null_mut(),
            handler: T::default(),
        };

        // プラグイン名をコピー
        let name = format!("{}\0", plugin_name);
        for (i, b) in name.bytes().enumerate() {
            if i < kMaxStringLength {
                plugin.fPluginName[i] = b as i8;
            }
        }

        plugin
    }

    /// スイートが取得されているか確認
    pub fn SuitesAcquired(&self) -> bool {
        !self.fSuites.is_null()
    }

    /// プラグインの起動処理
    pub fn StartupPlugin(&mut self, message: *mut SPInterfaceMessage) -> ASErr {
        let mut error = kNoErr;

        if error == kNoErr {
            error = self.SetGlobal();
        }

        // Suitesの作成
        self.fSuites = match Suites::new() {
            Ok(suites) => Box::into_raw(suites),
            Err(err) => {
                error = err;
                null_mut()
            }
        };

        if !self.fSuites.is_null() && error == kNoErr {
            unsafe {
                error = (*self.fSuites).Error();
                (*self.fSuites).InitializeRefCount();
            }

            // エラーがあればSuitesを解放
            if error != kNoErr {
                unsafe {
                    let _ = Box::from_raw(self.fSuites);
                    self.fSuites = null_mut();
                }
            }
        }

        if error == kNoErr {
            unsafe {
                error = sSPPlugins.SetPluginName((*message).d.self_, self.fPluginName.as_ptr());

                if error == kNoErr {
                    let mut notifier_name = [0i8; kMaxStringLength];
                    // sprintf相当の処理
                    let name_str = format!("{} App Started Notifier\0", self.get_plugin_name_str());
                    copy_cstr_to_buffer(&name_str, &mut notifier_name);

                    error = sAINotifier.AddNotifier(
                        (*message).d.self_,
                        notifier_name.as_ptr(),
                        kAIApplicationStartedNotifier,
                        &mut self.fApplicationStartedNotifier as *mut _ as *mut c_void
                    );
                }

                if error == kNoErr {
                    let mut notifier_name = [0i8; kMaxStringLength];
                    // sprintf相当の処理
                    let name_str = format!("{} Application Shutdown Notifier\0", self.get_plugin_name_str());
                    copy_cstr_to_buffer(&name_str, &mut notifier_name);

                    error = sAINotifier.AddNotifier(
                        (*message).d.self_,
                        notifier_name.as_ptr(),
                        kAIApplicationShutdownNotifier,
                        &mut self.fApplicationShutdownNotifer as *mut _ as *mut c_void
                    );
                }
            }
        }

        if error == kNoErr {
            error = self.AllocateSuiteTables();
        }

        if error == kNoErr {
            self.FillSuiteTables();
        }

        if error == kNoErr {
            error = self.LockPlugin(true);
        }

        error
    }

    /// プラグインの終了処理
    pub fn ShutdownPlugin(&mut self, _message: *mut SPInterfaceMessage) -> ASErr {
        let error = kNoErr;

        // Suitesの解放
        if !self.fSuites.is_null() {
            unsafe {
                let _ = Box::from_raw(self.fSuites);
                self.fSuites = null_mut();
            }
        }

        error
    }

    /// プラグインのアンロード処理
    pub fn UnloadPlugin(&mut self, _message: *mut SPInterfaceMessage) -> ASErr {
        let error = kNoErr;

        self.EmptySuiteTables();

        if !self.fSuites.is_null() {
            unsafe {
                let _ = Box::from_raw(self.fSuites);
                self.fSuites = null_mut();
            }
        }

        error
    }

    /// プラグインのリロード処理
    pub fn ReloadPlugin(&mut self, _message: *mut SPInterfaceMessage) -> ASErr {
        let mut error = kNoErr;

        if error == kNoErr {
            error = self.SetGlobal();
        }

        // suites が null なら新しく作成
        if self.fSuites.is_null() {
            self.fSuites = match Suites::new() {
                Ok(suites) => Box::into_raw(suites),
                Err(err) => {
                    error = err;
                    null_mut()
                }
            };

            if !self.fSuites.is_null() && error == kNoErr {
                unsafe {
                    error = (*self.fSuites).Error();
                    (*self.fSuites).InitializeRefCount();
                }

                if error != kNoErr {
                    unsafe {
                        let _ = Box::from_raw(self.fSuites);
                        self.fSuites = null_mut();
                    }
                }
            }
        }

        if error == kNoErr {
            self.FillSuiteTables();
        }

        error
    }

    /// メッセージがリロードメッセージかチェック
    pub fn IsReloadMsg(caller: *const c_char, selector: *const c_char) -> bool {
        unsafe {
            // C文字列を比較
            strcmp(caller, kSPAccessCaller.as_ptr()) == 0 &&
            strcmp(selector, kSPAccessReloadSelector.as_ptr()) == 0
        }
    }

    /// メッセージ処理 - プラグインのコア機能
    pub fn Message(&mut self, caller: *const c_char, selector: *const c_char, message: *mut c_void) -> ASErr {
        // オプショナルスイートの取得
        self.AcquireOptionalSuites();

        let mut error = kUnhandledMsgErr;

        unsafe {
            // Sweet Pea メッセージ
            if strcmp(caller, kSPAccessCaller.as_ptr()) == 0 {
                if strcmp(selector, kSPAccessUnloadSelector.as_ptr()) == 0 {
                    error = self.UnloadPlugin(message as *mut SPInterfaceMessage);
                } else if strcmp(selector, kSPAccessReloadSelector.as_ptr()) == 0 {
                    error = self.ReloadPlugin(message as *mut SPInterfaceMessage);
                }
            } else if strcmp(caller, kSPInterfaceCaller.as_ptr()) == 0 {
                if strcmp(selector, kSPInterfaceAboutSelector.as_ptr()) == 0 {
                    error = kNoErr;
                } else if strcmp(selector, kSPInterfaceStartupSelector.as_ptr()) == 0 {
                    error = kNoErr;
                }
            } else if strcmp(caller, kSPCacheCaller.as_ptr()) == 0 {
                if strcmp(selector, kSPPluginPurgeCachesSelector.as_ptr()) == 0 {
                    if self.Purge() {
                        error = kSPPluginCachesFlushResponse;
                    } else {
                        error = kSPPluginCouldntFlushResponse;
                    }
                }
            } else if strcmp(caller, kSPPropertiesCaller.as_ptr()) == 0 {
                if strcmp(selector, kSPPropertiesAcquireSelector.as_ptr()) == 0 {
                    error = self.handler.AcquireProperty(message as *mut SPPropertiesMessage);
                } else if strcmp(selector, kSPPropertiesReleaseSelector.as_ptr()) == 0 {
                    error = self.handler.ReleaseProperty(message as *mut SPPropertiesMessage);
                }
            }

            // アプリケーション通知
            else if strcmp(caller, kCallerAINotify.as_ptr()) == 0 {
                let msg = message as *mut AINotifierMessage;

                if (*msg).notifier == self.fApplicationStartedNotifier {
                    error = self.handler.PostStartupPlugin();
                } else if (*msg).notifier == self.fApplicationShutdownNotifer {
                    error = self.handler.PreShutdownPlugin();
                }

                if error == kNoErr || error == kUnhandledMsgErr {
                    if strcmp(selector, kSelectorAINotify.as_ptr()) == 0 {
                        error = self.handler.Notify(msg);
                    }
                }
            }

            // AIPluginトレイトの他のイベントハンドラを呼び出し
            else {
                // AIPluginトレイトのdispatch_messageを呼び出す
                error = AIPlugin::dispatch_message(&mut self.handler, caller, selector, message);
            }
        }

        error
    }

    /// グローバルプラグイン参照を設定
    pub fn SetGlobal(&self) -> ASErr {
        // グローバルインスタンスが設定されていることを確認
        kNoErr
    }

    /// オプショナルスイートの取得
    pub fn AcquireOptionalSuites(&mut self) -> ASErr {
        if !self.fSuites.is_null() {
            unsafe {
                (*self.fSuites).acquire_Optional_Suites();
            }
        }
        kNoErr
    }

    /// エラー報告
    pub fn ReportError(&mut self, error: ASErr, _caller: *const c_char, _selector: *const c_char, _message: *mut c_void) {
        if Self::FilterError(error) {
            return;
        }

        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(duration) => duration.as_secs() as i64,
            Err(_) => 0,
        };

        if error == self.fLastError && self.fSupressDuplicateErrors &&
           now < self.fLastErrorTime + self.fErrorTimeout {
            return;
        }

        self.fLastError = error;
        self.fLastErrorTime = now;
        Self::DefaultError(self.fPluginRef, error);
    }

    /// デフォルトエラーハンドリング
    pub fn DefaultError(ref_: SPPluginRef, error: ASErr) {
        if Self::FilterError(error) {
            return;
        }

        unsafe {
            let mut got_basic = false;
            if sAIUser.is_null() {
                if sSPBasic.is_null() {
                    return;
                }

                let err = (*sSPBasic).AcquireSuite(
                    kAIUserSuite.as_ptr(),
                    kAIUserSuiteVersion,
                    &mut sAIUser as *mut _ as *mut *const c_void
                );

                if err != 0 || sAIUser.is_null() {
                    return;
                }

                got_basic = true;
            }

            let mut msg = [0i8; 128];
            let m = Self::FindMsg(ref_, error, &mut msg);

            if m.is_null() {
                if got_basic {
                    (*sSPBasic).ReleaseSuite(kAIUserSuite.as_ptr(), kAIUserSuiteVersion);
                    sAIUser = null_mut();
                }
                return;
            }

            let mut mbuf = [0i8; 128];

            if CStr::from_ptr(m).to_bytes().len() < 120 {
                let mut err_string = [0i8; 10];

                if error < 16385 {  // 単純な数値エラーの場合
                    let err_str = format!("{}\0", error);
                    copy_cstr_to_buffer(&err_str, &mut err_string);
                } else {  // 4バイト文字列エラー
                    for i in (0..=3).rev() {
                        err_string[i] = (error & 0xff) as i8;
                        error = error >> 8;
                    }
                    err_string[4] = 0;
                }

                copy_cstr_to_buffer(CStr::from_ptr(m).to_str().unwrap_or("Error"), &mut mbuf);

                m = mbuf.as_ptr();
            }

            // Unicode文字列に変換してエラー表示
            let unicode_str = ai::UnicodeString::new(CStr::from_ptr(m).to_str().unwrap_or("Error"));
            (*sAIUser).ErrorAlert(unicode_str);

            if got_basic {
                (*sSPBasic).ReleaseSuite(kAIUserSuite.as_ptr(), kAIUserSuiteVersion);
                sAIUser = null_mut();
            }
        }
    }

    /// エラーメッセージを検索
    pub fn FindMsg(_ref: SPPluginRef, _error: ASErr, _buf: &mut [i8]) -> *const i8 {
        null_mut()
    }

    /// 特定のエラーをフィルタリング
    pub fn FilterError(error: ASErr) -> bool {
        static ERRORS: [ASErr; 18] = [
            kUnknownFormatErr,
            kRefusePluginGroupReply,
            kWantsAfterMsgPluginGroupReply,
            kMarkValidPluginGroupReply,
            kDontCarePluginGroupReply,
            kDestroyPluginGroupReply,
            kCheckPluginGroupReply,
            kCustomHitPluginGroupReply,
            kToolCantTrackCursorErr,
            kSPPluginCachesFlushResponse,
            kSPSuiteNotFoundError,
            kSPCantAcquirePluginError,
            kDidSymbolReplacement,
            kSkipEditGroupReply,
            kIterationCanQuitReply,
            kCanceledErr,
            361,
            0  // 終端マーカー
        ];

        ERRORS.iter().any(|&e| e == error)
    }

    /// プラグイン名を取得するヘルパーメソッド（内部用）
    pub fn get_plugin_name_str(&self) -> String {
        let cstr = unsafe { CStr::from_ptr(self.fPluginName.as_ptr()) };
        cstr.to_str().unwrap_or("Plugin").to_string()
    }

    /// プラグインロック処理
    pub fn LockPlugin(&mut self, lock: bool) -> ASErr {
        if lock {
            self.fLockCount += 1;
            if self.fLockCount == 1 {
                unsafe {
                    (*sSPAccess).AcquirePlugin(self.fPluginRef, &mut self.fPluginAccess);
                }
            }
        } else {
            self.fLockCount -= 1;
            if self.fLockCount == 0 {
                unsafe {
                    (*sSPAccess).ReleasePlugin(self.fPluginAccess);
                    self.fPluginAccess = null_mut();
                }
            } else if self.fLockCount < 0 {
                self.fLockCount = 0;  // 本来は起きないはずだが念のため
            }
        }
        kNoErr
    }

    /// その他のPlugin固有メソッド
    pub fn AllocateSuiteTables(&mut self) -> ASErr { kNoErr }
    pub fn FillSuiteTables(&mut self) -> ASErr { kNoErr }
    pub fn EmptySuiteTables(&mut self) -> ASErr { kNoErr }
    pub fn Purge(&self) -> bool { false }
}

/// C文字列をi8バッファにコピーするヘルパー関数
pub fn copy_cstr_to_buffer(src: &str, dst: &mut [i8]) {
    let bytes = src.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if i < dst.len() {
            dst[i] = b as i8;
        }
    }
}


/// 型変換用のモジュール
pub mod ai {
    use std::ffi::CStr;

    #[repr(C)]
    pub struct UnicodeString {
        // 必要なフィールド
    }

    impl UnicodeString {
        pub fn new(_str: &str) -> Self {
            Self { /* 初期化 */ }
        }
    }
}

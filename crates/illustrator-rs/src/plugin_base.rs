use crate::AIPlugin;
use crate::ai_suites::AISuite;

/// マクロ `define_plugin!` は Adobe Illustrator プラグインのエントリーポイントを定義します。
///
/// このマクロは、`AIPlugin` トレイトを実装した構造体を受け取り、
/// C API との互換性を持つプラグインを生成します。
///
/// # 引数
/// * `$plugin_type` - `AIPlugin` トレイトを実装した構造体の型
/// * `$plugin_name` - プラグインの名前（文字列リテラル）
/// * `$suites` - オプション: 必要なスイートの配列 `suites = [AISuite::DocumentSuite, AISuite::RasterSuite]`
///
/// # 例
///
/// ```
/// use illustrator::AIPlugin;
/// use illustrator::ai_suites::AISuite;
///
/// #[derive(Default)]
/// struct MyPlugin {
///     // プラグイン固有のフィールド
/// }
///
/// impl AIPlugin for MyPlugin {
///     // イベントハンドラの実装
///     fn Notify(&mut self, message: *mut AINotifierMessage) -> ASErr {
///         // 通知イベントの処理
///         0
///     }
/// }
///
/// // 基本形式
/// illustrator::define_plugin!(MyPlugin, "My Illustrator Plugin");
///
/// // スイート指定形式
/// illustrator::define_plugin!(
///     MyPlugin,
///     name = "My Illustrator Plugin",
///     suites = [AISuite::DocumentSuite, AISuite::RasterSuite]
/// );
/// ```
#[macro_export]
macro_rules! define_plugin {
    // 基本形式: プラグイン型と名前のみ
    ($plugin_type:ty, $plugin_name:expr) => {
        define_plugin!($plugin_type, name = $plugin_name, suites = []);
    };

    // 拡張形式: プラグイン型、名前、スイート配列
    ($plugin_type:ty, name = $plugin_name:expr, suites = [$($suite:expr),*]) => {
        /// C言語関数のbindgen実装
        extern "C" {
          pub static mut sSPBasic: *mut SPBasicSuite;
          pub static mut sAIUser: *mut AIUserSuite;
          pub static mut sSPPlugins: *mut SPPluginsSuite;
          pub static mut sAINotifier: *mut AINotifierSuite;
          pub static mut sSPAccess: *mut  SPAccessSuite;
        }

        // グローバルプラグインインスタンス
        static mut PLUGIN_INSTANCE: Option<Box<$crate::plugin_impl::Plugin<$plugin_type>>> = None;

        // 必要なスイートを定義
        static REQUIRED_SUITES: &'static [$crate::ai_suites::AISuite] = &[$($suite),*];

        /// プラグインのエントリーポイント
        #[no_mangle]
        pub extern "C" fn AllocatePlugin(plugin_ref: $crate::adobe_bindings::SPPluginRef) -> *mut $crate::plugin_impl::Plugin<$plugin_type> {
            let plugin = Box::new($crate::plugin_impl::Plugin::<$plugin_type>::new(plugin_ref, $plugin_name));
            unsafe {
                PLUGIN_INSTANCE = Some(plugin);
                PLUGIN_INSTANCE.as_mut().unwrap() as *mut _
            }
        }

        #[no_mangle]
        pub extern "C" fn FixupReload(_plugin: *mut $crate::plugin_impl::Plugin<$plugin_type>) {
            // リロード処理後の修正処理
        }

        #[no_mangle]
        pub extern "C" fn PluginMain(
            caller: *mut std::ffi::c_char,
            selector: *mut std::ffi::c_char,
            message: *mut std::ffi::c_void
        ) -> $crate::adobe_bindings::ASErr {
            use $crate::plugin_impl::*;
            use $crate::adobe_bindings::*;
            use std::ptr::null_mut;

            let mut error = kNoErr;
            let msg_data = unsafe { &mut *(message as *mut SPMessageData) };

            let plugin = msg_data.globals as *mut Plugin<$plugin_type>;

            unsafe { sSPBasic = msg_data.basic; }

            unsafe {
                // C文字列を比較
                if strcmp(caller, kSPInterfaceCaller.as_ptr()) == 0 {
                    if strcmp(selector, kSPInterfaceStartupSelector.as_ptr()) == 0 {
                        let new_plugin = AllocatePlugin(msg_data.self_);
                        if !new_plugin.is_null() {
                            msg_data.globals = new_plugin as *mut std::ffi::c_void;
                            error = (*new_plugin).StartupPlugin(message as *mut SPInterfaceMessage);

                            if error != kNoErr {
                                // スタートアップに失敗した場合は解放
                                PLUGIN_INSTANCE = None;
                                msg_data.globals = null_mut();
                            }
                        } else {
                            error = kOutOfMemoryErr;
                        }
                    } else if strcmp(selector, kSPInterfaceShutdownSelector.as_ptr()) == 0 {
                        if !plugin.is_null() {
                            error = (*plugin).ShutdownPlugin(message as *mut SPInterfaceMessage);
                            PLUGIN_INSTANCE = None;
                            msg_data.globals = null_mut();
                        }
                    }
                }

                if !plugin.is_null() {
                    if Plugin::<$plugin_type>::IsReloadMsg(caller, selector) {
                        // virtual関数を呼び出す前にこれを呼ぶ（Message関数など）
                        FixupReload(plugin);
                        error = (*plugin).ReloadPlugin(message as *mut SPInterfaceMessage);
                    } else {
                        // ロードやリロードがスイート取得に失敗していた場合、保護する
                        if (*plugin).SuitesAcquired() {
                            error = (*plugin).Message(caller, selector, message);
                        } else {
                            error = kNoErr;
                        }
                    }

                    if error == kUnhandledMsgErr {
                        error = kNoErr;
                    }
                }

                if error != 0 {
                    if !plugin.is_null() {
                        (*plugin).ReportError(error, caller, selector, message);
                    } else {
                        Plugin::<$plugin_type>::DefaultError(msg_data.self_, error);
                    }
                }
            }

            error
        }
    }
}

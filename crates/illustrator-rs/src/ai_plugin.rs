use std::ffi::c_void;
use illustrator_sys::ai_sys::*;

pub trait AIPlugin {
    // プラグイン基本イベント
    fn PostStartupPlugin(&mut self) -> ASErr { kNoErr }
    fn PreShutdownPlugin(&mut self) -> ASErr { kNoErr }

    // プロパティ関連
    fn AcquireProperty(&mut self, _message: *mut SPPropertiesMessage) -> ASErr { kUnhandledMsgErr }
    fn ReleaseProperty(&mut self, _message: *mut SPPropertiesMessage) -> ASErr { kUnhandledMsgErr }

    // 通知処理
    fn Notify(&mut self, _message: *mut AINotifierMessage) -> ASErr { kNoErr }

    // アクション
    fn GoAction(&mut self, _message: *mut DoActionMessage) -> ASErr { kNoErr }

    // メニュー
    fn GoMenuItem(&mut self, _message: *mut AIMenuMessage) -> ASErr { kUnhandledMsgErr }
    fn UpdateMenuItem(&mut self, _message: *mut AIMenuMessage) -> ASErr { kUnhandledMsgErr }

    // フィルター
    fn GetFilterParameters(&mut self, _message: *mut AIFilterMessage) -> ASErr { kUnhandledMsgErr }
    fn GoFilter(&mut self, _message: *mut AIFilterMessage) -> ASErr { kUnhandledMsgErr }

    // プラグイングループ
    fn PluginGroupNotify(&mut self, _message: *mut AIPluginGroupMessage) -> ASErr { kUnhandledMsgErr }
    fn PluginGroupUpdate(&mut self, _message: *mut AIPluginGroupMessage) -> ASErr { kUnhandledMsgErr }

    // ファイルフォーマット
    fn GetFileFormatParameters(&mut self, _message: *mut AIFileFormatMessage) -> ASErr { kUnhandledMsgErr }
    fn GoFileFormat(&mut self, _message: *mut AIFileFormatMessage) -> ASErr { kUnhandledMsgErr }
    fn CheckFileFormat(&mut self, _message: *mut AIFileFormatMessage) -> ASErr { kUnhandledMsgErr }
    fn FileFormatUpdate(&mut self, _message: *mut AIUpdateFileFormatMessage) -> ASErr { kUnhandledMsgErr }
    fn SetFileFormatParameters(&mut self, _message: *mut DoActionMessage) -> ASErr { kUnhandledMsgErr }

    // ツール関連
    fn EditTool(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn TrackToolCursor(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn ToolMouseDown(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn ToolMouseDrag(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn ToolMouseUp(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn SelectTool(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn DeselectTool(&mut self, _message: *mut AIToolMessage) -> ASErr { kUnhandledMsgErr }
    fn ReselectTool(&mut self, _message: *mut AIToolMessage) -> ASErr { kNoErr }
    fn DecreaseDiameter(&mut self, _message: *mut AIToolMessage) -> ASErr { kNoErr }
    fn IncreaseDiameter(&mut self, _message: *mut AIToolMessage) -> ASErr { kNoErr }

    // ライブエフェクト
    fn EditLiveEffectParameters(&mut self, _message: *mut AILiveEffectEditParamMessage) -> ASErr { kUnhandledMsgErr }
    fn GoLiveEffect(&mut self, _message: *mut AILiveEffectGoMessage) -> ASErr { kUnhandledMsgErr }
    fn LiveEffectInterpolate(&mut self, _message: *mut AILiveEffectInterpParamMessage) -> ASErr { kUnhandledMsgErr }
    fn LiveEffectGetInputType(&mut self, _message: *mut AILiveEffectInputTypeMessage) -> ASErr { kUnhandledMsgErr }
    fn LiveEffectScaleParameters(&mut self, _message: *mut AILiveEffectScaleParamMessage) -> ASErr { kUnhandledMsgErr }
    fn LiveEffectConvertColorSpace(&mut self, _message: *mut AILiveEffectConvertColorMessage) -> ASErr { kUnhandledMsgErr }
    fn LiveEffectAdjustColors(&mut self, _message: *mut AILiveEffectAdjustColorsMessage) -> ASErr { kUnhandledMsgErr }
    fn LiveEffectHandleMerge(&mut self, _message: *mut AILiveEffectHandleMergeMessage) -> ASErr { kUnhandledMsgErr }

    // タイマー
    fn GoTimer(&mut self, _message: *mut AITimerMessage) -> ASErr { kUnhandledMsgErr }

    // クリップボード
    fn GoClipboard(&mut self, _message: *mut AIClipboardMessage) -> ASErr { kUnhandledMsgErr }
    fn CanCopyClipboard(&mut self, _message: *mut AIClipboardMessage) -> ASErr { kUnhandledMsgErr }
    fn CloneClipboard(&mut self, _message: *mut AIClipboardMessage) -> ASErr { kUnhandledMsgErr }
    fn DisposeClipboard(&mut self, _message: *mut AIClipboardMessage) -> ASErr { kUnhandledMsgErr }

    // ワークスペース
    fn WorkspaceWrite(&mut self, _message: *mut AIWorkspaceMessage) -> ASErr { kUnhandledMsgErr }
    fn WorkspaceRestore(&mut self, _message: *mut AIWorkspaceMessage) -> ASErr { kUnhandledMsgErr }
    fn WorkspaceDefault(&mut self, _message: *mut AIWorkspaceMessage) -> ASErr { kUnhandledMsgErr }

    // メッセージディスパッチのヘルパーメソッド - 各イベントハンドラを呼び出す
    fn dispatch_message(&mut self, caller: *const c_char, selector: *const c_char, message: *mut c_void) -> ASErr {
        unsafe {
            if strcmp(caller, kCallerAINotify.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAINotify.as_ptr()) == 0 {
                    return self.Notify(message as *mut AINotifierMessage);
                }
            }
            else if strcmp(caller, kActionCaller.as_ptr()) == 0 {
                if strcmp(selector, kDoActionSelector.as_ptr()) == 0 {
                    return self.GoAction(message as *mut DoActionMessage);
                }
            }
            else if strcmp(caller, kCallerAIMenu.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIGoMenuItem.as_ptr()) == 0 {
                    return self.GoMenuItem(message as *mut AIMenuMessage);
                }
                else if strcmp(selector, kSelectorAIUpdateMenuItem.as_ptr()) == 0 {
                    return self.UpdateMenuItem(message as *mut AIMenuMessage);
                }
            }
            else if strcmp(caller, kCallerAIFilter.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIGetFilterParameters.as_ptr()) == 0 {
                    return self.GetFilterParameters(message as *mut AIFilterMessage);
                }
                else if strcmp(selector, kSelectorAIGoFilter.as_ptr()) == 0 {
                    return self.GoFilter(message as *mut AIFilterMessage);
                }
            }
            else if strcmp(caller, kCallerAIPluginGroup.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAINotifyEdits.as_ptr()) == 0 {
                    return self.PluginGroupNotify(message as *mut AIPluginGroupMessage);
                }
                else if strcmp(selector, kSelectorAIUpdateArt.as_ptr()) == 0 {
                    return self.PluginGroupUpdate(message as *mut AIPluginGroupMessage);
                }
            }
            else if strcmp(caller, kCallerAIFileFormat.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIGetFileFormatParameters.as_ptr()) == 0 {
                    return self.GetFileFormatParameters(message as *mut AIFileFormatMessage);
                }
                else if strcmp(selector, kSelectorAIGoFileFormat.as_ptr()) == 0 {
                    return self.GoFileFormat(message as *mut AIFileFormatMessage);
                }
                else if strcmp(selector, kSelectorAICheckFileFormat.as_ptr()) == 0 {
                    return self.CheckFileFormat(message as *mut AIFileFormatMessage);
                }
                else if strcmp(selector, kSelectorAIUpdateFileFormat.as_ptr()) == 0 {
                    return self.FileFormatUpdate(message as *mut AIUpdateFileFormatMessage);
                }
                else if strcmp(selector, kDoActionSelector.as_ptr()) == 0 {
                    return self.SetFileFormatParameters(message as *mut DoActionMessage);
                }
            }
            else if strcmp(caller, kCallerAITool.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIEditToolOptions.as_ptr()) == 0 {
                    return self.EditTool(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAITrackToolCursor.as_ptr()) == 0 {
                    return self.TrackToolCursor(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIToolMouseDown.as_ptr()) == 0 {
                    return self.ToolMouseDown(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIToolMouseDrag.as_ptr()) == 0 {
                    return self.ToolMouseDrag(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIToolMouseUp.as_ptr()) == 0 {
                    return self.ToolMouseUp(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAISelectTool.as_ptr()) == 0 {
                    return self.SelectTool(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIDeselectTool.as_ptr()) == 0 {
                    return self.DeselectTool(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIReselectTool.as_ptr()) == 0 {
                    return self.ReselectTool(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIToolDecreaseDiameter.as_ptr()) == 0 {
                    return self.DecreaseDiameter(message as *mut AIToolMessage);
                }
                else if strcmp(selector, kSelectorAIToolIncreaseDiameter.as_ptr()) == 0 {
                    return self.IncreaseDiameter(message as *mut AIToolMessage);
                }
            }
            else if strcmp(caller, kCallerAILiveEffect.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIEditLiveEffectParameters.as_ptr()) == 0 {
                    return self.EditLiveEffectParameters(message as *mut AILiveEffectEditParamMessage);
                }
                else if strcmp(selector, kSelectorAIGoLiveEffect.as_ptr()) == 0 {
                    return self.GoLiveEffect(message as *mut AILiveEffectGoMessage);
                }
                else if strcmp(selector, kSelectorAILiveEffectInterpolate.as_ptr()) == 0 {
                    return self.LiveEffectInterpolate(message as *mut AILiveEffectInterpParamMessage);
                }
                else if strcmp(selector, kSelectorAILiveEffectInputType.as_ptr()) == 0 {
                    return self.LiveEffectGetInputType(message as *mut AILiveEffectInputTypeMessage);
                }
                else if strcmp(selector, kSelectorAILiveEffectScaleParameters.as_ptr()) == 0 {
                    return self.LiveEffectScaleParameters(message as *mut AILiveEffectScaleParamMessage);
                }
                else if strcmp(selector, kSelectorAILiveEffectConverColorSpace.as_ptr()) == 0 {
                    return self.LiveEffectConvertColorSpace(message as *mut AILiveEffectConvertColorMessage);
                }
                else if strcmp(selector, kSelectorAILiveEffectAdjustColors.as_ptr()) == 0 {
                    return self.LiveEffectAdjustColors(message as *mut AILiveEffectAdjustColorsMessage);
                }
                else if strcmp(selector, kSelectorAILiveEffectHandleMerge.as_ptr()) == 0 {
                    return self.LiveEffectHandleMerge(message as *mut AILiveEffectHandleMergeMessage);
                }
            }
            else if strcmp(caller, kCallerAITimer.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIGoTimer.as_ptr()) == 0 {
                    return self.GoTimer(message as *mut AITimerMessage);
                }
            }
            else if strcmp(caller, kCallerAIClipboard.as_ptr()) == 0 {
                if strcmp(selector, kSelectorAIGoClipboard.as_ptr()) == 0 {
                    return self.GoClipboard(message as *mut AIClipboardMessage);
                }
                else if strcmp(selector, kSelectorAICanCopyClipboard.as_ptr()) == 0 {
                    return self.CanCopyClipboard(message as *mut AIClipboardMessage);
                }
                else if strcmp(selector, kSelectorAICloneClipboard.as_ptr()) == 0 {
                    return self.CloneClipboard(message as *mut AIClipboardMessage);
                }
                else if strcmp(selector, kSelectorAIDisposeClipboard.as_ptr()) == 0 {
                    return self.DisposeClipboard(message as *mut AIClipboardMessage);
                }
            }
            else if strcmp(caller, kAIWorkspaceCaller.as_ptr()) == 0 {
                if strcmp(selector, kAIWSWriteSelector.as_ptr()) == 0 {
                    return self.WorkspaceWrite(message as *mut AIWorkspaceMessage);
                }
                else if strcmp(selector, kAIWSRestoreSelector.as_ptr()) == 0 {
                    return self.WorkspaceRestore(message as *mut AIWorkspaceMessage);
                }
                else if strcmp(selector, kAIWSDefaultSelector.as_ptr()) == 0 {
                    return self.WorkspaceDefault(message as *mut AIWorkspaceMessage);
                }
            }
        }

        kUnhandledMsgErr
    }
}

// 標準スイートをインポート
extern "C" {
    fn strcmp(s1: *const c_char, s2: *const c_char) -> i32;
}

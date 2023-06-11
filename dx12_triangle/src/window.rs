//エラー取得用 module
#[path = "./dx12error.rs"]
mod dx12error;
use dx12error::Dx12Error;

//レンダリング処理用 module
#[path = "./renderer.rs"]
mod renderer;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows::{
    core::*, Win32::Foundation::*, Win32::System::LibraryLoader::*,
    Win32::UI::WindowsAndMessaging::*,
};

/// window_procedure
///
/// ウィンドウメッセージをアプリケーション内で振り分けるための通関手続きを行う関数
/// 言い換えるならば,OSから送られてきたメッセージを処理するためのコールバック関数
///
/// # Arguments
/// *  'hwnd' - window handle
/// *  'msg' - message
/// *  'wparam' - wparam
/// *  'lparam' - lparam
///
/// # Returns
/// *  'LRESULT' - result
///
extern "system" fn window_procedure(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            // ウィンドウが破棄された場合アプリケーションを終了
            WM_DESTROY => {
                PostQuitMessage(0);
                LRESULT::default()
            }
            // その他のメッセージはデフォルトの処理。
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// window 処理構造体
/// # Fields
/// *  'hwnd' - window handle
///
pub struct Window {
    hwnd: HWND,
}

impl Window {
    /// ウィンドウ作成
    ///
    /// # Arguments
    /// *  'app_name' - application name
    /// *  'width' - window size width
    /// *  'height' - window size height
    ///
    /// # Returns
    /// *  'Ok(Window)' - window handle
    /// *  'Err(Dx12Error)' - error message
    pub fn create_window(
        app_name: &str,
        width: i32,
        height: i32,
    ) -> std::result::Result<Window, Dx12Error> {
        //インスタンス（アプリケーション）へのハンドル
        let instance: HMODULE = match unsafe { GetModuleHandleA(None) } {
            Ok(handle) => handle,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed Get Module handleA: {:?}",
                    err
                )))
            }
        };

        //アプリケーション名をワイド文字に変換
        let app_name_wide: Vec<u16> = OsStr::new(app_name).encode_wide().chain(Some(0)).collect();

        //ウィンドウクラスのパラメータを設定
        let wc = WNDCLASSEXW {
            cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_CLASSDC,
            lpfnWndProc: Some(window_procedure),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance,
            lpszClassName: PCWSTR(app_name_wide.as_ptr()),
            ..Default::default()
        };

        //ウィンドウクラス登録
        let atom = unsafe { RegisterClassExW(&wc) };
        debug_assert_ne!(atom, 0);

        //シザリング初期化
        let mut window_rect = RECT {
            left: 0,
            top: 0,
            right: width,
            bottom: height,
        };

        //シザリング登録
        if unsafe { AdjustWindowRect(&mut window_rect, WS_OVERLAPPEDWINDOW, false) }.as_bool()
            != true
        {
            return Err(Dx12Error::new("Failed to adjust window rect"));
        }

        //ウィンドウ作成
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                PCWSTR(app_name_wide.as_ptr()),
                PCWSTR(app_name_wide.as_ptr()),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                window_rect.right - window_rect.left,
                window_rect.bottom - window_rect.top,
                None,
                None,
                instance,
                None,
            )
        };

        //ウィンドウ表示
        unsafe { ShowWindow(hwnd, SW_SHOW) };

        Ok(Window { hwnd: hwnd })
    }

    /// メッセージループ処理
    ///
    /// # Arguments
    /// *  'resource' - renderer::MainRenderingResources
    ///
    /// # Returns
    /// *  'Ok(())' - success
    /// *  'Err(Dx12Error)' - error message
    ///
    pub fn process_messages(
        &mut self,
        resource: &mut crate::renderer::MainRenderingResources,
    ) -> std::result::Result<(), Dx12Error> {
        loop {
            let mut msg = MSG::default();
            //メッセージが存在するか確認，存在する場合msgに格納
            if unsafe { PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE) }.into() {
                unsafe {
                    //キーボード入力メッセージ（主にキーの押下とリリース）を文字メッセージに変換
                    TranslateMessage(&msg);
                    //取得したメッセージを対応するウィンドウプロシージャに送信
                    DispatchMessageA(&msg);
                }

                //アプリケーションが終わる時にmessageがWM_QUITになる
                if msg.message == WM_QUIT {
                    break;
                }

                let clear_color = [1.0, 0.0, 0.0, 1.0];

                //描画初期処理
                match resource.begin_render(clear_color) {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(Dx12Error::new(&format!(
                            "Failed to begin render: {:?}",
                            err
                        )))
                    }
                }

                //TODO:　ここに描画処理を記述

                //描画初期処理
                match resource.end_render() {
                    Ok(_) => (),
                    Err(err) => {
                        return Err(Dx12Error::new(&format!("Failed to end render: {:?}", err)))
                    }
                }
            } else {
                continue;
            }
        }

        Ok(())
    }
}

/// window メンバ変数取得 関数群
impl Window {
    /// window ハンドル 取得
    pub fn get_hwnd(&self) -> HWND {
        self.hwnd
    }
}

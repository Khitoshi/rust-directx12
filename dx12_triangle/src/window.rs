#[path = "../src/dx12error.rs"]
mod dx12error;
use dx12error::Dx12Error;

use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;

use windows::{
    core::*, Win32::Foundation::*, Win32::Graphics::Direct3D::Fxc::*, Win32::Graphics::Direct3D::*,
    Win32::Graphics::Direct3D12::*, Win32::Graphics::Dxgi::Common::*,
    Win32::Graphics::Dxgi::IDXGIFactory6, Win32::Graphics::Dxgi::*,
    Win32::System::LibraryLoader::*, Win32::System::Threading::*,
    Win32::UI::WindowsAndMessaging::*,
};
//ウィンドウプロシージャ

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

pub struct Window {
    hwnd: HWND,
    app_name_wide: Vec<u16>,
}

impl Window {
    //自身のCreate関数
    pub fn new(
        app_name: &str,
        window_rect_right: i32,
        window_rect_bottom: i32,
    ) -> std::result::Result<Window, Dx12Error> {
        let instance: HMODULE = match unsafe { GetModuleHandleA(None) } {
            Ok(handle) => handle,
            Err(err) => {
                return Err(Dx12Error::new(&format!(
                    "Failed Get Module handleA: {:?}",
                    err
                )))
            }
        };

        let app_name_wide: Vec<u16> = OsStr::new(app_name).encode_wide().chain(Some(0)).collect();
        //ウィンドウクラスのパラメータを設定
        let wc: WNDCLASSEXW = WNDCLASSEXW {
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
            right: window_rect_right,
            bottom: window_rect_bottom,
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

        Ok(Window {
            hwnd: hwnd,
            app_name_wide: app_name_wide,
        })
    }

    //メッセージループ処理
    pub fn process_messages_loop(&mut self) {
        loop {
            let mut msg: MSG = MSG::default();
            if unsafe { PeekMessageA(&mut msg, None, 0, 0, PM_REMOVE) }.into() {
                unsafe {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }

                if msg.message == WM_QUIT {}
            }
        }
    }
}

impl Window {
    pub fn get_hwnd(&self) -> HWND {
        return self.hwnd;
    }
}

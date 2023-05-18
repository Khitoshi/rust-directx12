use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::{HBRUSH, HWND};
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::wingdi::{GetStockObject, BLACK_BRUSH};
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, CW_USEDEFAULT, MSG, WM_DESTROY, WNDCLASSW,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;

//ウィンドウプロシージャ
extern "system" fn window_procedure(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            // ウィンドウが破棄された場合アプリケーションを終了
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            // その他のメッセージはデフォルトの処理。
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub struct Window {
    hwnd: HWND,
}

impl Window {
    pub fn new() -> Result<Self, &'static str> {
        // Create the window and return a new Window struct
        //ウィンドウクラスを識別名設定
        let window_class_name = to_wstring("view window");

        //ウィンドウの見た目、動作を設定
        let h_instance = unsafe { GetModuleHandleW(null_mut()) };
        let hbr_background = unsafe { GetStockObject(BLACK_BRUSH as i32) as HBRUSH };
        let window_class = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(window_procedure),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: h_instance,
            hIcon: null_mut(),
            hCursor: null_mut(),
            hbrBackground: hbr_background,
            lpszMenuName: null_mut(),
            lpszClassName: window_class_name.as_ptr(),
        };

        //ウィンドウクラス登録
        let result = unsafe { RegisterClassW(&window_class) };
        if result == 0 {
            return Err("Failed to register the window class.");
        }

        //ウィンドウ作成
        let hwnd = unsafe {
            CreateWindowExW(
                0,
                window_class_name.as_ptr(),
                to_wstring("Hello, world!").as_ptr(),
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                null_mut(),
                null_mut(),
                GetModuleHandleW(null_mut()),
                null_mut(),
            )
        };
        if hwnd.is_null() {
            return Err("Failed to create the window.");
        }

        Ok(Window { hwnd: hwnd })
    }

    pub fn get_hwnd(&self) -> HWND {
        self.hwnd
    }

    pub fn process_messages(&mut self) -> Result<bool, &'static str> {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            match GetMessageW(&mut msg, self.hwnd, 0, 0) {
                -1 => Err("An error occurred while getting message."),
                0 => Ok(false),
                _ => {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                    Ok(true)
                }
            }
        }
    }
}

//文字列をnull終端のUTF-16文字列に変換
fn to_wstring(string: &str) -> Vec<u16> {
    OsStr::new(string).encode_wide().chain(once(0)).collect()
}

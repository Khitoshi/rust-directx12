extern crate winapi;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, GetMessageW, PostQuitMessage, RegisterClassW,
    TranslateMessage, DispatchMessageW, MSG, WNDCLASSW, WS_OVERLAPPEDWINDOW, WS_VISIBLE,
    CW_USEDEFAULT, WM_DESTROY
};
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::iter::once;
use std::ptr::null_mut;

// ウィンドウプロシージャを定義します。これはウィンドウが受け取るメッセージを処理します。
extern "system" fn window_procedure(
    hwnd: HWND,
    msg: UINT,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            // ウィンドウが破棄されたときに、アプリケーションを終了します。
            WM_DESTROY => {
                PostQuitMessage(0);
                0
            }
            // 他のメッセージはデフォルトの処理に任せます。
            _ => DefWindowProcW(hwnd, msg, wparam, lparam) ,
        }
    }
}

// Rustの文字列をnull終端のUTF-16文字列に変換します。これはWinAPI関数に文字列を渡すために必要です。
fn to_wstring(string: &str) -> Vec<u16> {
    OsStr::new(string).encode_wide().chain(once(0)).collect()
}

fn main() {
    //ウィンドウクラスを識別するための名前
    let window_class_name = to_wstring("view window");

    // ウィンドウクラスを定義します。これはウィンドウの見た目と動作を決定します。
    let window_class = WNDCLASSW {
        style: 0,
        lpfnWndProc: Some(window_procedure),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: unsafe { GetModuleHandleW(null_mut()) },
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null_mut(),
        lpszClassName: window_class_name.as_ptr(),
    };

    unsafe {
        // ウィンドウクラスを登録します。
        RegisterClassW(&window_class);

        // ウィンドウを作成します。
        let hwnd = CreateWindowExW(
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
        );

        // メッセージループを開始します。これはユーザーからの入力を処理します。
        let mut msg: MSG = std::mem::zeroed();

        while GetMessageW(&mut msg, hwnd, 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}
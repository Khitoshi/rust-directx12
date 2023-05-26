use std::ffi::OsStr;
use std::iter::once;
use std::os::windows::ffi::OsStrExt;
use std::ptr::null_mut;
use winapi::shared::minwindef::{LPARAM, LRESULT, UINT, WPARAM};
use winapi::shared::windef::HWND;
use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::winuser::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, PostQuitMessage,
    RegisterClassW, TranslateMessage, CW_USEDEFAULT, MSG, WM_DESTROY, WNDCLASSW,
    WS_OVERLAPPEDWINDOW, WS_VISIBLE,
};

//ウィンドウメッセージをアプリケーション内で振り分けるための通関手続きを行う
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
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

// Rustの文字列をnull終端のUTF-16文字列に変換
fn to_wstring(string: &str) -> Vec<u16> {
    OsStr::new(string).encode_wide().chain(once(0)).collect()
}

fn main() {
    //ウィンドウクラスを識別するための名前
    let window_class_name = to_wstring("view window");

    // これはウィンドウの見た目と動作を設定するウィンドウクラスの設定
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
        //ウィンドウクラス登録
        RegisterClassW(&window_class);

        //ウィンドウ作成dwExStyle
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

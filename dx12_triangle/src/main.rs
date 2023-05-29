mod renderer;
use renderer::Dx12Resources;
mod window;

use window::Window;
use windows::Win32::Foundation::HWND;

fn main() {
    let mut hwnd: HWND;
    let app_name = "triangle test";
    let window_rect_right: u64 = 1080;
    let window_rect_bottom: u32 = 720;

    let mut window = match Window::new(
        app_name,
        window_rect_right as i32,
        window_rect_bottom as i32,
    ) {
        Ok(window) => window,
        Err(err) => {
            return;
        }
    };

    /*
    // Dx12Resourcesの作成
    let mut dx12_resources =
    match Dx12Resources::new(hwnd, window_rect_right, window_rect_bottom) {
        Ok(resouce) => resouce,
        Err(err)=>{
            return ;}
        };
        */

    window.process_messages_loop();
}

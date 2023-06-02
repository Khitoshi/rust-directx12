#[path = "../src/renderer.rs"]
mod renderer;
use renderer::MainRenderingResources;

#[path = "../src/window.rs"]
mod window;

fn main() {
    let app_name = "triangle test";
    let window_rect_right: u64 = 1080;
    let window_rect_bottom: u32 = 720;

    let mut window = match window::Window::new(
        app_name,
        window_rect_right as i32,
        window_rect_bottom as i32,
    ) {
        Ok(window) => window,
        Err(err) => {
            println!("error!:{}", err.get_message());
            return;
        }
    };

    let mut dx12_resources =
        match MainRenderingResources::new(window.get_hwnd(), window_rect_right, window_rect_bottom)
        {
            Ok(resource) => resource,
            Err(err) => {
                println!("error!:{}", err);
                return;
            }
        };

    window.process_messages_loop(&mut dx12_resources);
}

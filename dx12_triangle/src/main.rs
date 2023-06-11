mod renderer;
mod window;

fn main() {
    //app title name
    let app_name = "triangle test";

    //window size
    let window_rect_width: u64 = 1080;
    let window_rect_height: u32 = 720;

    //window 作成
    let mut window = match window::Window::create_window(
        app_name,
        window_rect_width as i32,
        window_rect_height as i32,
    ) {
        Ok(window) => window,
        Err(err) => {
            println!("error!:{}", err.get_message());
            return;
        }
    };

    //リソース生成
    let mut dx12_resources = match renderer::MainRenderingResources::create(
        window.get_hwnd(),
        window_rect_width,
        window_rect_height,
    ) {
        Ok(resource) => resource,
        Err(err) => {
            println!("error!:{}", err);
            return;
        }
    };

    //メッセージループ処理
    match window.process_messages(&mut dx12_resources) {
        Ok(_) => print!("success!"),
        Err(err) => {
            println!("error!:{}", err);
            return;
        }
    }
}

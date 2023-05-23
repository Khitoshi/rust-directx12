mod renderer;
mod window;

fn main() {
    let mut win = match window::Window::new() {
        Ok(win) => win,
        Err(err) => {
            println!("Failed to create a window: {}", err);
            return;
        }
    };

    // Dx12Resourcesの作成
    let mut dx12_resources = match Dx12Resources::new() {
        Ok(resources) => resources,
        Err(err) => {
            println!("Failed to initialize DirectX 12 resources: {}", err);
            return;
        }
    };

    loop {
        match win.process_messages() {
            Ok(continue_loop) => {
                if !continue_loop {
                    break;
                }

                // ここでレンダリングやその他のタスクを行う
                match dx12_resources.render() {
                    Ok(_) => {}
                    Err(err) => {
                        println!("Failed to render: {}", err);
                        break;
                    }
                }
            }
            Err(err) => {
                println!("Failed to process messages: {}", err);
                break;
            }
        }

        // Do other tasks such as rendering here.
    }
}

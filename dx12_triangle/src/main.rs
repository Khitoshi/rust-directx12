mod window;

fn main() {
    let mut win = match window::Window::new() {
        Ok(win) => win,
        Err(err) => {
            println!("Failed to create a window: {}", err);
            return;
        }
    };

    loop {
        match win.process_messages() {
            Ok(continue_loop) => {
                if !continue_loop {
                    break;
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

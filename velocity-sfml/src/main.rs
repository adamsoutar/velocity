use velocity_core::constants::*;
use velocity_core::shell_layer::get_shell_layer;

use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;

const WINDOW_WIDTH: u32 = 960;
const WINDOW_HEIGHT: u32 = 750;

fn main() {
    let style = Style::RESIZE | Style::TITLEBAR | Style::CLOSE;
    let mut window = RenderWindow::new(
        (WINDOW_WIDTH, WINDOW_HEIGHT),
        "velocity",
        style,
        &Default::default(),
    );

    let mut shell_layer = get_shell_layer();
    let mut buffer = [0; FD_BUFFER_SIZE_BYTES];
    let mut written_bytes = 0;
    loop {
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                }
                _ => {}
            }
        }

        shell_layer.read(&mut buffer, &mut written_bytes);
        if written_bytes > 0 {
            let s = String::from_utf8_lossy(&buffer[..written_bytes]);
            println!("{}", s);
        }

        window.display();
    }
}

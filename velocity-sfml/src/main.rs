use velocity_core::constants::*;
use velocity_core::tty::TtyState;

use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;

const WINDOW_WIDTH: u32 = 1120;
const WINDOW_HEIGHT: u32 = 600;
const FONT_SIZE: u32 = 24;

fn main() {
    let style = Style::RESIZE | Style::TITLEBAR | Style::CLOSE;
    let mut window = RenderWindow::new(
        (WINDOW_WIDTH, WINDOW_HEIGHT),
        "velocity",
        style,
        &Default::default(),
    );

    // TODO: Customisable fonts
    let font = Font::from_file("/System/Library/Fonts/Menlo.ttc").unwrap();
    let font_width = font.glyph(65, FONT_SIZE, false, 0.).advance;

    let mut tty = TtyState::new();
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

        window.clear(Color::BLACK);

        tty.read();
        for i in 0..tty.size.rows {
            let row_id = tty.scrollback_start + i;

            // This line is blank as of yet
            if tty.scrollback_buffer.len() <= row_id {
                continue;
            }

            let row_string: String = tty.scrollback_buffer[row_id].iter().collect();
            let mut row_text = Text::new(&row_string[..], &font, FONT_SIZE);
            row_text.set_position(Vector2f::new(0., i as f32 * FONT_SIZE as f32));
            window.draw(&row_text);
        }

        // Cursor
        let mut cursor_block =
            RectangleShape::with_size(Vector2f::new(font_width, FONT_SIZE as f32));
        cursor_block.set_fill_color(Color::WHITE);
        cursor_block.set_position(Vector2f::new(
            tty.cursor_pos.x as f32 * font_width,
            tty.cursor_pos.y as f32 * FONT_SIZE as f32,
        ));
        window.draw(&cursor_block);

        window.display();
    }
}

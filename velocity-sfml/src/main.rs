use colours::terminal_colour_to_sfml_colour;
use velocity_core::tty::TtyState;

use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;

mod colours;

const FONT_SIZE: u32 = 24;

const COLUMNS: usize = 80;
const ROWS: usize = 25;

fn main() {
    // iTerm uses "Monaco"
    let font = Font::from_file("/System/Library/Fonts/Monaco.ttf").unwrap();
    // TODO: Loop through all the characters widths and choose the largest - or the
    //   space advance if that's bigger
    let font_width = font.glyph(32, FONT_SIZE, false, 0.).advance();
    // This doesn't seem to be defined anywhere concrete in SFML at all, but I've checked
    // a few fonts and this is coincidentally right for everything I've tested
    let font_height = FONT_SIZE as f32 * 1.2;

    let window_width = (font_width * COLUMNS as f32) as u32;
    let window_height = (font_height * ROWS as f32) as u32;

    let style = Style::RESIZE | Style::TITLEBAR | Style::CLOSE;
    let mut window = RenderWindow::new(
        (window_width, window_height),
        "velocity",
        style,
        &Default::default(),
    );

    let mut tty = TtyState::new(COLUMNS, ROWS);
    loop {
        while let Some(ev) = window.poll_event() {
            match ev {
                Event::Closed => {
                    window.close();
                    return;
                }
                // NOTE: "system" is the Super key
                Event::KeyPressed {
                    code: _code,
                    alt: _alt,
                    ctrl: _ctrl,
                    shift: _shift,
                    system: _system,
                } => {
                    // TODO: Handle things like ^C here
                }
                Event::TextEntered { unicode } => {
                    let mut buffer = [0; 4];
                    let bytes = unicode.encode_utf8(&mut buffer).as_bytes();
                    tty.write(&bytes);
                }
                _ => {}
            }
        }

        // TODO: If the text has a certain background colour, and then the screen is cleared,
        //   we should change the whole background colour.
        window.clear(Color::BLACK);

        tty.read();
        for i in 0..tty.size.rows {
            let row_id = tty.scrollback_start + i;

            // This line is blank as of yet
            if tty.scrollback_buffer.len() <= row_id {
                continue;
            }

            for l in 0..tty.scrollback_buffer[row_id].len() {
                let letter = tty.scrollback_buffer[row_id][l];
                if letter.style.invisible {
                    continue;
                }

                let mut char_text = Text::new(&letter.char.to_string(), &font, FONT_SIZE);
                char_text
                    .set_position(Vector2f::new(l as f32 * font_width, i as f32 * font_height));

                // The SFML TextStyle system is a bitmask.
                let mut sfml_text_style: sfml::graphics::TextStyle =
                    sfml::graphics::TextStyle::REGULAR;
                if letter.style.bold {
                    sfml_text_style |= sfml::graphics::TextStyle::BOLD
                }
                if letter.style.italic {
                    sfml_text_style |= sfml::graphics::TextStyle::ITALIC
                }
                if letter.style.strikethrough {
                    sfml_text_style |= sfml::graphics::TextStyle::STRIKETHROUGH
                }
                if letter.style.underlined {
                    sfml_text_style |= sfml::graphics::TextStyle::UNDERLINED
                }
                char_text.set_style(sfml_text_style);

                char_text.set_fill_color(terminal_colour_to_sfml_colour(letter.style.foreground));

                window.draw(&char_text);
            }
        }

        // Cursor
        // TODO: Does text foreground colour colour the cursor?
        //   If it does, we can make TtyState's text_style public
        let mut cursor_block = RectangleShape::with_size(Vector2f::new(font_width, font_height));
        cursor_block.set_fill_color(Color::WHITE);
        cursor_block.set_position(Vector2f::new(
            tty.cursor_pos.x as f32 * font_width,
            tty.cursor_pos.y as f32 * font_height,
        ));
        window.draw(&cursor_block);

        window.display();
    }
}

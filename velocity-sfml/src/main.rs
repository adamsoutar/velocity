use colours::terminal_colour_to_sfml_colour;
use colours::DefaultColourVersion;
use velocity_core::constants::special_characters::ESCAPE;
use velocity_core::tty::TtyState;

use phf::phf_map;
use sfml::graphics::*;
use sfml::system::*;
use sfml::window::*;

mod colours;

const FONT_SIZE: u32 = 24;

const COLUMNS: usize = 80;
const ROWS: usize = 25;

static SPECIAL_KEYS: phf::Map<u8, u8> = phf_map! {
    // Right arrow is SFML 71
    71u8 => 68,
    72u8 => 67,
    73u8 => 65,
    74u8 => 66,
};

fn main() {
    // TODO: Less font hardcoding. Eg, some Linux users might have their fonts in a different
    //   place. Can we ask the system where fonts are?
    #[cfg(target_os = "macos")]
    let font_path = "/System/Library/Fonts/Monaco.ttf";
    #[cfg(target_os = "linux")]
    let font_path = "/usr/share/fonts/truetype/noto/NotoMono-Regular.ttf";

    let font = Font::from_file(font_path).unwrap();
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
                    code,
                    alt: _alt,
                    ctrl,
                    shift: _shift,
                    system: _system,
                } => {
                    let key_number = code as isize;

                    if SPECIAL_KEYS.contains_key(&(key_number as u8)) {
                        // TODO: If "Application Cursor Keys" is enabled, the middle byte
                        //   should be 0x4F, not 91
                        let buffer = [ESCAPE as u8, 91, SPECIAL_KEYS[&(key_number as u8)]];
                        tty.write(&buffer);
                    }

                    if key_number >= 0 && key_number <= 25 {
                        // These are alphabetical keys
                        // We don't support anything else at the moment
                        if ctrl {
                            // This maps C to ^C etc.
                            let buffer = [key_number as u8 + 1];
                            tty.write(&buffer);
                        }
                    }
                }
                Event::TextEntered { unicode } => {
                    let mut buffer = [0; 4];
                    let mut bytes = unicode.encode_utf8(&mut buffer).as_bytes();

                    // If the user presses enter, SFML sends us a line feed, but
                    // the shell expects a carriage return
                    if bytes.len() == 1 && bytes[0] == '\n' as u8 {
                        bytes = &['\r' as u8];
                    }

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

                let char_pos = Vector2f::new(l as f32 * font_width, i as f32 * font_height);

                let mut fg_colour = terminal_colour_to_sfml_colour(
                    letter.style.foreground,
                    DefaultColourVersion::Foreground,
                );
                let mut bg_colour = terminal_colour_to_sfml_colour(
                    letter.style.background,
                    DefaultColourVersion::Background,
                );
                if letter.style.invisible {
                    fg_colour = bg_colour;
                }
                if letter.style.reverse_video {
                    std::mem::swap(&mut fg_colour, &mut bg_colour)
                }

                // First, draw the background behind the character
                let mut bg = RectangleShape::with_size(Vector2f::new(font_width, font_height));
                bg.set_fill_color(bg_colour);
                bg.set_position(char_pos);
                window.draw(&bg);

                let mut char_text = Text::new(&letter.char.to_string(), &font, FONT_SIZE);
                char_text.set_position(char_pos);

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

                char_text.set_fill_color(fg_colour);

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

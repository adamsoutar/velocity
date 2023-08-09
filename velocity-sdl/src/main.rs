use colours::{terminal_colour_to_sdl_colour, DefaultColourVersion};
use phf::phf_map;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;
use sdl2::ttf::FontStyle;
use std::thread;
use std::time::Duration;

use velocity_core::tty::TtyState;

mod colours;

const FONT_SIZE: u16 = 24;

const COLUMNS: usize = 80;
const ROWS: usize = 25;

// static SPECIAL_KEYS: phf::Map<u8, u8> = phf_map! {
//     // Right arrow is SFML 71
//     71u8 => 68,
//     72u8 => 67,
//     73u8 => 65,
//     74u8 => 66,
// };

macro_rules! rect(
    ($x:expr, $y:expr, $w:expr, $h:expr) => (
        Rect::new($x as i32, $y as i32, $w as u32, $h as u32)
    )
);

pub fn main() {
    // TODO: Less font hardcoding. Eg, some Linux users might have their fonts in a different
    //   place. Can we ask the system where fonts are?
    #[cfg(target_os = "macos")]
    let font_path = "/System/Library/Fonts/Monaco.ttf";
    #[cfg(target_os = "linux")]
    let font_path = "/usr/share/fonts/truetype/noto/NotoMono-Regular.ttf";

    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();

    let window = video_subsystem
        .window("Velocity", 800, 600)
        .position_centered()
        .allow_highdpi()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let texture_creator = canvas.texture_creator();
    let mut font = ttf_context.load_font(font_path, FONT_SIZE).unwrap();

    let space_surface = font
        .render(" ")
        .blended(Color::RGBA(255, 255, 255, 255))
        .unwrap();
    let space_texture = texture_creator
        .create_texture_from_surface(&space_surface)
        .unwrap();
    let TextureQuery {
        width: space_width,
        height: space_height,
        ..
    } = space_texture.query();

    let window_width = space_width * COLUMNS as u32;
    let window_height = space_height * ROWS as u32;

    let (win_width, _) = canvas.window().size();
    let (drw_width, _) = canvas.window().drawable_size();
    let dpi_multiplier = drw_width as f32 / win_width as f32;
    canvas
        .window_mut()
        .set_size(
            (window_width as f32 / dpi_multiplier) as u32,
            (window_height as f32 / dpi_multiplier) as u32,
        )
        .unwrap();

    canvas.present();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let mut tty = TtyState::new(COLUMNS, ROWS);
    'running: loop {
        canvas.set_draw_color(Color::RGB(0, 0, 0));
        canvas.clear();

        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::TextInput { text, .. } => {
                    let bytes = text.into_bytes();
                    tty.write(&bytes);
                }
                Event::KeyDown {
                    keycode,
                    scancode,
                    keymod,
                    repeat,
                    ..
                } => {
                    println!(
                        "Keycode: {:?}, Scancode: {:?}, Keymod: {:?}, Repeat: {:?}",
                        keycode, scancode, keymod, repeat
                    );
                    if keycode == Some(Keycode::Backspace) {
                        tty.write(&[8]);
                    }
                    if keycode == Some(Keycode::Return) {
                        tty.write(&['\r' as u8]);
                    }
                }
                _ => {}
            }
        }

        tty.read();

        for i in 0..tty.size.rows {
            let row_id = tty.scrollback_start + i;

            // This line is blank as of yet
            if tty.scrollback_buffer.len() <= row_id {
                continue;
            }

            for l in 0..tty.scrollback_buffer[row_id].len() {
                let letter = tty.scrollback_buffer[row_id][l];

                let mut fg_colour = terminal_colour_to_sdl_colour(
                    letter.style.foreground,
                    DefaultColourVersion::Foreground,
                );
                let mut bg_colour = terminal_colour_to_sdl_colour(
                    letter.style.background,
                    DefaultColourVersion::Background,
                );
                if letter.style.invisible {
                    fg_colour = bg_colour;
                }
                if letter.style.reverse_video {
                    std::mem::swap(&mut fg_colour, &mut bg_colour)
                }

                // The SDL2 TextStyle system is a bitmask.
                let mut sdl_text_style: FontStyle = FontStyle::NORMAL;
                if letter.style.bold {
                    sdl_text_style |= FontStyle::BOLD
                }
                if letter.style.italic {
                    sdl_text_style |= FontStyle::ITALIC
                }
                if letter.style.strikethrough {
                    sdl_text_style |= FontStyle::STRIKETHROUGH
                }
                if letter.style.underlined {
                    sdl_text_style |= FontStyle::UNDERLINE
                }
                font.set_style(sdl_text_style);

                let char_texture = font
                    .render(&letter.char.to_string())
                    .blended(fg_colour)
                    .unwrap()
                    .as_texture(&texture_creator)
                    .unwrap();
                // let char_texture = texture_creator
                //     .create_texture_from_surface(&char_surface)
                //     .unwrap();
                // char_surface.
                let TextureQuery {
                    width: char_width,
                    height: char_height,
                    ..
                } = char_texture.query();
                let char_rect = rect!(
                    l as u32 * space_width,
                    i as u32 * space_height,
                    char_width,
                    char_height
                );

                // First, draw the background behind the character
                canvas.set_draw_color(bg_colour);
                canvas.fill_rect(char_rect).unwrap();
                // Then the character itself
                canvas.copy(&char_texture, None, Some(char_rect)).unwrap();
            }
        }

        let cursor_rect = rect!(
            tty.cursor_pos.x as u32 * space_width,
            tty.cursor_pos.y as u32 * space_height,
            space_width,
            space_height
        );
        canvas.set_draw_color(Color::WHITE);
        canvas.fill_rect(cursor_rect).unwrap();

        canvas.present();
    }
}

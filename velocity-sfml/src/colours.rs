use velocity_core::escape_sequence::sequence::TerminalColour;

use sfml::graphics::Color;

// This is iTerm2's default colour palette
const BLACK: Color = Color::BLACK;
const RED: Color = Color::rgb(201, 27, 0);
const GREEN: Color = Color::rgb(0, 194, 0);
const YELLOW: Color = Color::rgb(199, 196, 0);
const BLUE: Color = Color::rgb(2, 37, 199);
const MAGENTA: Color = Color::rgb(201, 48, 199);
const CYAN: Color = Color::rgb(0, 197, 199);
const WHITE: Color = Color::rgb(199, 199, 199);

const BRIGHT_BLACK: Color = Color::rgb(103, 103, 103);
const BRIGHT_RED: Color = Color::rgb(255, 109, 103);
const BRIGHT_GREEN: Color = Color::rgb(95, 249, 103);
const BRIGHT_YELLOW: Color = Color::rgb(254, 251, 103);
const BRIGHT_BLUE: Color = Color::rgb(104, 113, 255);
const BRIGHT_MAGENTA: Color = Color::rgb(255, 118, 255);
const BRIGHT_CYAN: Color = Color::rgb(95, 253, 255);
const BRIGHT_WHITE: Color = Color::rgb(255, 254, 254);

#[derive(PartialEq)]
pub enum DefaultColourVersion {
    Foreground,
    Background,
}

pub fn terminal_colour_to_sfml_colour(
    c: TerminalColour,
    default_colour_version: DefaultColourVersion,
) -> Color {
    match c {
        TerminalColour::Black => BLACK,
        TerminalColour::Red => RED,
        TerminalColour::Green => GREEN,
        TerminalColour::Yellow => YELLOW,
        TerminalColour::Blue => BLUE,
        TerminalColour::Magenta => MAGENTA,
        TerminalColour::Cyan => CYAN,
        TerminalColour::White => WHITE,
        TerminalColour::BrightBlack => BRIGHT_BLACK,
        TerminalColour::BrightRed => BRIGHT_RED,
        TerminalColour::BrightGreen => BRIGHT_GREEN,
        TerminalColour::BrightYellow => BRIGHT_YELLOW,
        TerminalColour::BrightBlue => BRIGHT_BLUE,
        TerminalColour::BrightMagenta => BRIGHT_MAGENTA,
        TerminalColour::BrightCyan => BRIGHT_CYAN,
        TerminalColour::BrightWhite => BRIGHT_WHITE,
        TerminalColour::Default => {
            if default_colour_version == DefaultColourVersion::Foreground {
                WHITE
            } else {
                BLACK
            }
        }
    }
}

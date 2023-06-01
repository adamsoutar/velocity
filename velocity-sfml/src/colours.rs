use velocity_core::escape_sequence::sequence::TerminalColour;

use sfml::graphics::Color;

pub fn terminal_colour_to_sfml_colour(c: TerminalColour) -> Color {
    let mut as_num = c as usize;
    // For convenience's sake, treat background colours as foreground colours
    // They're the same colour, anyway.
    if as_num > 40 && c != TerminalColour::DefaultBackground {
        as_num -= 10;
    }

    match as_num {
        49 => Color::BLACK,
        39 => Color::WHITE,
        30 => Color::BLACK,
        31 => Color::RED,
        32 => Color::GREEN,
        33 => Color::YELLOW,
        34 => Color::BLUE,
        35 => Color::MAGENTA,
        36 => Color::CYAN,
        37 => Color::WHITE,
        _ => panic!(
            "SFML colour requested for {:?}. That's not a valid decoration colour.",
            c
        ),
    }
}

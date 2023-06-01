use crate::escape_sequence::sequence::{EscapeSequence, SGRCode, TerminalColour};

#[derive(Clone, Copy)]
pub enum BlinkingMode {
    None,
    Slow,
    Rapid,
}

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub bold: bool,
    pub faint: bool,
    pub italic: bool,
    pub underlined: bool,
    pub blinking: BlinkingMode,
    pub reverse_video: bool,
    pub invisible: bool,
    pub strikethrough: bool,
    pub foreground: TerminalColour,
    pub background: TerminalColour,
}

impl TextStyle {
    fn apply_basic_colour_setter(&mut self, sgr: &SGRCode) {
        let as_num = *sgr as usize;

        if as_num >= 30 && as_num <= 37 {
            // This is a foreground setter
            self.foreground = num::FromPrimitive::from_usize(as_num - 30).unwrap();
            return;
        }
        if as_num >= 40 && as_num <= 47 {
            // This is a background setter
            self.background = num::FromPrimitive::from_usize(as_num - 40).unwrap();
            return;
        }
        if as_num >= 90 && as_num <= 97 {
            // This is a bright foreground setter
            // Only -80 not 90 because bright numbers start at 10
            self.foreground = num::FromPrimitive::from_usize(as_num - 80).unwrap();
            return;
        }
        if as_num >= 100 && as_num <= 107 {
            // This is a bright background setter
            // Only -80 not 90 because bright numbers start at 10
            self.background = num::FromPrimitive::from_usize(as_num - 90).unwrap();
            return;
        }
        if *sgr == SGRCode::SelectDefaultForegroundColour {
            self.foreground = TerminalColour::Default;
            return;
        }
        if *sgr == SGRCode::SelectDefaultBackgroundColour {
            self.background = TerminalColour::Default;
            return;
        }
    }

    fn apply_sgr_code(&mut self, sgr: &SGRCode) {
        if sgr_code_is_a_basic_colour_setter(sgr) {
            self.apply_basic_colour_setter(sgr);
            return;
        }

        match sgr {
            SGRCode::ResetAllTextStyles => {
                self.bold = false;
                self.faint = false;
                self.italic = false;
                self.underlined = false;
                self.blinking = BlinkingMode::None;
                self.reverse_video = false;
                self.invisible = false;
                self.strikethrough = false;
                self.foreground = TerminalColour::Default;
                self.background = TerminalColour::Default;
            }
            SGRCode::EnableBoldText => self.bold = true,
            SGRCode::EnableFaintText => self.faint = true,
            SGRCode::EnableItalicText => self.italic = true,
            SGRCode::EnableUnderlinedText => self.underlined = true,
            SGRCode::EnableRapidBlinkingText => self.blinking = BlinkingMode::Rapid,
            SGRCode::EnableSlowBlinkingText => self.blinking = BlinkingMode::Slow,
            SGRCode::EnableReverseVideoMode => self.reverse_video = true,
            SGRCode::EnableInvisibleText => self.invisible = true,
            SGRCode::EnableStrikethroughText => self.strikethrough = true,

            SGRCode::ResetTextWeight => {
                self.bold = false;
                self.faint = false;
            }
            SGRCode::ResetItalicText => self.italic = false,
            SGRCode::ResetUnderlinedText => self.underlined = false,
            SGRCode::ResetBlinkingText => self.blinking = BlinkingMode::None,
            SGRCode::ResetInverseVideoMode => self.reverse_video = false,
            SGRCode::ResetInvisibleText => self.invisible = false,
            SGRCode::ResetStrikethroughText => self.strikethrough = false,

            _ => {
                println!("Unimplemented SGR code: {:?}", sgr)
            }
        }
    }

    pub fn apply_escape_sequence(&mut self, sequence: &EscapeSequence) {
        match sequence {
            EscapeSequence::SelectGraphicRendition(sgr_codes) => {
                for sgr in sgr_codes {
                    self.apply_sgr_code(sgr)
                }
            }
            // Nothing else affects text style at the moment
            _ => (),
        }
    }

    pub fn new() -> TextStyle {
        TextStyle {
            bold: false,
            faint: false,
            italic: false,
            underlined: false,
            blinking: BlinkingMode::None,
            reverse_video: false,
            invisible: false,
            strikethrough: false,
            foreground: TerminalColour::Default,
            background: TerminalColour::Default,
        }
    }
}

fn sgr_code_is_a_basic_colour_setter(sgr: &SGRCode) -> bool {
    match *sgr as usize {
        // Normal foreground and background, then bright foreground, then bright background
        30..=49 | 90..=97 | 100..=107 => true,
        _ => false,
    }
}

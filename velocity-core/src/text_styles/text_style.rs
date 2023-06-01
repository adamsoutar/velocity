use crate::escape_sequence::sequence::{EscapeSequence, TerminalColour};

#[derive(Clone, Copy)]
pub struct TextStyle {
    pub bold: bool,
    pub faint: bool,
    pub italic: bool,
    pub underlined: bool,
    pub blinking: bool,
    pub reverse_video: bool,
    pub invisible: bool,
    pub strikethrough: bool,
    pub foreground: TerminalColour,
    pub background: TerminalColour,
}

impl TextStyle {
    pub fn apply_escape_sequence(&mut self, sequence: &EscapeSequence) {
        match sequence {
            EscapeSequence::EnableBoldText => self.bold = true,
            EscapeSequence::EnableFaintText => self.faint = true,
            EscapeSequence::EnableItalicText => self.italic = true,
            EscapeSequence::EnableUnderlinedText => self.underlined = true,
            EscapeSequence::EnableBlinkingText => self.blinking = true,
            EscapeSequence::EnableReverseVideoMode => self.reverse_video = true,
            EscapeSequence::EnableInvisibleText => self.invisible = true,
            EscapeSequence::EnableStrikethroughText => self.strikethrough = true,
            EscapeSequence::DisableBoldText => self.bold = false,
            EscapeSequence::DisableFaintText => self.faint = false,
            EscapeSequence::DisableItalicText => self.italic = false,
            EscapeSequence::DisableUnderlinedText => self.underlined = false,
            EscapeSequence::DisableBlinkingText => self.blinking = false,
            EscapeSequence::DisableReverseVideoMode => self.reverse_video = false,
            EscapeSequence::DisableInvisibleText => self.invisible = false,
            EscapeSequence::DisableStrikethroughText => self.strikethrough = false,
            EscapeSequence::ResetAllTextStyles => {
                self.bold = false;
                self.faint = false;
                self.italic = false;
                self.underlined = false;
                self.blinking = false;
                self.reverse_video = false;
                self.invisible = false;
                self.strikethrough = false;
                self.foreground = TerminalColour::DefaultForeground;
                self.background = TerminalColour::DefaultBackground;
            }
            EscapeSequence::SetTextColour(colour_args) => {
                self.apply_escape_sequence(&*colour_args.initial_sub_sequence);
                for colour in &colour_args.colour_sequence {
                    if (*colour as usize) == 0 {
                        self.apply_escape_sequence(&EscapeSequence::ResetAllTextStyles);
                        continue;
                    }

                    if (*colour as usize) < 40 {
                        // Colours below 40 are foreground colours
                        self.foreground = *colour;
                    } else {
                        // Colours above 40 are background colours
                        self.background = *colour;
                    }
                }
            }
            // Any other escape sequence is not related to text
            _ => {}
        }
    }

    pub fn new() -> TextStyle {
        TextStyle {
            bold: false,
            faint: false,
            italic: false,
            underlined: false,
            blinking: false,
            reverse_video: false,
            invisible: false,
            strikethrough: false,
            foreground: TerminalColour::DefaultForeground,
            background: TerminalColour::DefaultBackground,
        }
    }
}

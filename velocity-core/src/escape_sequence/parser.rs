use crate::constants::special_characters::*;

use super::sequence::{EscapeSequence, EscapeSequence::*, TerminalColour, TextColourArgs};

#[derive(PartialEq)]
enum SequenceType {
    Undetermined, // Don't know yet (just ESC so far)
    CSI,          // Control Sequence Introducer (ESC followed by "[")
    DCS,          // Device Control String (ESC followed by "P")
    OSC,          // Operating System Command (ESC followed by "]")
    NonStandard,  // Special ones made up by other programmers (ESC followed by a space)
}

#[derive(Debug)]
pub enum SequenceFinished {
    // Return the finished sequece if we're ready to.
    // If this is None, we're done parsing because we gave up/don't support what the program
    // said to us.
    Yes(Option<EscapeSequence>),
    No,
}

pub struct EscapeSequenceParser {
    sequence_type: SequenceType,
    numeric_args: Vec<usize>,
    current_token: String,
}

impl EscapeSequenceParser {
    // Returns whether the sequence is over
    pub fn parse_character(&mut self, c: char) -> SequenceFinished {
        println!("Parsing '{}'", c);
        if self.sequence_type == SequenceType::Undetermined {
            self.sequence_type = match c {
                CONTROL_SEQUENCE_INTRODUCER => SequenceType::CSI,
                DEVICE_CONTROL_STRING => SequenceType::DCS,
                OPERATING_SYSTEM_COMMAND => SequenceType::OSC,
                SPACE => SequenceType::NonStandard,
                _ => {
                    println!("Unknown escape sequence introducer {:?}", c);
                    // We don't know how to parse this, so we're just going to call it a day.
                    return SequenceFinished::Yes(None);
                }
            };
            return SequenceFinished::No;
        }

        match self.sequence_type {
            SequenceType::CSI => self.parse_csi_character(c),
            // We haven't implement parsing for anything else yet
            _ => SequenceFinished::Yes(None),
        }
    }

    fn parse_csi_character(&mut self, c: char) -> SequenceFinished {
        if is_part_of_token(c) {
            self.current_token.push(c);
        } else {
            let maybe_token_as_num: Result<usize, _> = str::parse(&self.current_token[..]);
            self.current_token = String::new();

            if maybe_token_as_num.is_err() {
                // They gave us a nonsense number, give up
                return SequenceFinished::Yes(None);
            } else {
                self.numeric_args.push(maybe_token_as_num.unwrap())
            }

            // If it's ;, we'll fall through to the No
            if c != ';' {
                let final_sequence = self.parse_end_of_csi(c);
                return SequenceFinished::Yes(final_sequence);
            }
        }

        SequenceFinished::No
    }

    fn parse_end_of_csi(&self, c: char) -> Option<EscapeSequence> {
        // TODO: Actually implement this and do different things based on what we've parsed
        if c == 'm' {
            return self.parse_end_of_text_style_csi();
        }

        println!("Unsupported csi ending character '{}'", c);
        return None;
    }

    fn parse_end_of_text_style_csi(&self) -> Option<EscapeSequence> {
        if self.numeric_args.len() == 0 {
            // We don't understand this. You should tell us something about your desired
            // text style if you send this code.
            return None;
        }

        let mode_part = match self.numeric_args[0] {
            0 => ResetAllTextStyles,
            1 => EnableBoldText,
            22 => DisableBoldText, // TODO: DisableFaintText?
            2 => EnableFaintText,
            3 => EnableItalicText,
            23 => DisableItalicText,
            4 => EnableUnderlinedText,
            24 => DisableUnderlinedText,
            5 => EnableBlinkingText,
            25 => DisableBlinkingText,
            7 => EnableReverseVideoMode,
            27 => DisableReverseVideoMode,
            8 => EnableInvisibleText,
            28 => DisableInvisibleText,
            9 => EnableStrikethroughText,
            29 => DisableStrikethroughText,
            _ => {
                println!("Unsupported text style '{}'", self.numeric_args[0]);
                return None;
            }
        };

        if self.numeric_args.len() == 1 {
            Some(mode_part)
        } else {
            Some(SetTextColour(TextColourArgs {
                initial_sub_sequence: Box::new(mode_part),
                colour_sequence: self.numeric_args[1..]
                    .iter()
                    .map(num_to_terminal_colour)
                    .collect(),
            }))
        }
    }

    pub fn new() -> EscapeSequenceParser {
        EscapeSequenceParser {
            sequence_type: SequenceType::Undetermined,
            numeric_args: vec![],
            current_token: String::new(),
        }
    }
}

fn num_to_terminal_colour(n: &usize) -> TerminalColour {
    // TODO: Neater way to do this
    match *n {
        30 => TerminalColour::BlackForeground,
        40 => TerminalColour::BlackBackground,
        31 => TerminalColour::RedForeground,
        41 => TerminalColour::RedBackground,
        32 => TerminalColour::GreenForeground,
        42 => TerminalColour::GreenBackground,
        33 => TerminalColour::YellowForeground,
        43 => TerminalColour::YellowBackground,
        34 => TerminalColour::BlueForeground,
        44 => TerminalColour::BlueBackground,
        35 => TerminalColour::MagentaForeground,
        45 => TerminalColour::MagentaBackground,
        36 => TerminalColour::CyanForeground,
        46 => TerminalColour::CyanBackground,
        37 => TerminalColour::WhiteForeground,
        47 => TerminalColour::WhiteBackground,
        39 => TerminalColour::DefaultForeground,
        49 => TerminalColour::DefaultBackground,
        0 => TerminalColour::SpecialReset,
        _ => {
            println!("Unsupported TerminalColour code '{}'", n);
            TerminalColour::DefaultForeground
        }
    }
}

fn is_part_of_token(c: char) -> bool {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => true,
        _ => false,
    }
}

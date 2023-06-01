use crate::constants::special_characters::*;

use super::sequence::{EscapeSequence, SGRCode};

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
        if c == 'm' {
            return self.parse_end_of_text_style_csi();
        }

        println!("Unsupported csi ending character '{}'", c);
        return None;
    }

    fn parse_end_of_text_style_csi(&self) -> Option<EscapeSequence> {
        let mut sgr_codes: Vec<SGRCode> = vec![];

        for num in &self.numeric_args {
            let maybe_sgr = num::FromPrimitive::from_usize(*num);
            if maybe_sgr.is_none() {
                println!("Unknown SGR code {}", num);
            } else {
                sgr_codes.push(maybe_sgr.unwrap());
            }
        }

        Some(EscapeSequence::SelectGraphicRendition(sgr_codes))
    }

    pub fn new() -> EscapeSequenceParser {
        EscapeSequenceParser {
            sequence_type: SequenceType::Undetermined,
            numeric_args: vec![],
            current_token: String::new(),
        }
    }
}

fn is_part_of_token(c: char) -> bool {
    match c {
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' => true,
        _ => false,
    }
}

use crate::constants::special_characters::*;

use super::sequence::{EscapeSequence, SGRCode};

#[derive(PartialEq, Debug)]
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
    // These are based on the ECMA-48 Standard § 5.4
    csi_parameter_chars: Vec<char>,
    csi_intermediate_chars: Vec<char>,
}

impl EscapeSequenceParser {
    // Returns whether the sequence is over
    pub fn parse_character(&mut self, c: char) -> SequenceFinished {
        // println!("Parsing: {}", c);
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
            _ => {
                println!(
                    "Bailing from unimplemented escape SequenceType {:?}",
                    self.sequence_type
                );
                SequenceFinished::Yes(None)
            }
        }
    }

    fn parse_csi_character(&mut self, c: char) -> SequenceFinished {
        match c as usize {
            0x30..=0x3F => self.csi_parameter_chars.push(c),
            0x20..=0x2F => self.csi_intermediate_chars.push(c),
            0x40..=0x7E => return SequenceFinished::Yes(self.parse_csi_final_byte(c)),
            _ => {
                println!("Ignored unknown control sequence character '{}'", c);
            }
        }

        SequenceFinished::No
    }

    fn parse_csi_final_byte(&mut self, c: char) -> Option<EscapeSequence> {
        match c {
            'm' => self.parse_csi_select_graphic_rendition(),
            'K' => self.parse_csi_erase_in_line(),
            _ => {
                println!("Ignoring CSI due to unknown final byte '{}'", c);
                None
            }
        }
    }

    fn parse_csi_erase_in_line(&mut self) -> Option<EscapeSequence> {
        // This is the default value if there are no parameter bytes
        let mut erase_type = 0;

        if self.csi_parameter_chars.len() == 1 {
            erase_type = self.csi_parameter_chars[0]
                .to_string()
                .parse::<usize>()
                .unwrap_or_else(|err| {
                    println!(
                        "Error parsing CSI erase type '{}', returning Noop\n{:?}",
                        self.csi_parameter_chars[0], err
                    );
                    999
                });
        }

        let maybe_erase_type_enum = num::FromPrimitive::from_usize(erase_type);
        if maybe_erase_type_enum.is_none() {
            println!("Unknown CSI erase type '{}'", erase_type);
            return None;
        } else {
            return Some(EscapeSequence::EraseInLine(maybe_erase_type_enum.unwrap()));
        }
    }

    fn parse_csi_select_graphic_rendition(&mut self) -> Option<EscapeSequence> {
        let param_string: String = self.csi_parameter_chars.clone().into_iter().collect();
        let params: Vec<SGRCode> = param_string
            .split(';')
            .map(|sgr_code_str| {
                sgr_code_str.parse::<usize>().unwrap_or_else(|err| {
                    println!(
                        "Error parsing CSI SGR number '{}', transforming to Noop\n{:?}",
                        sgr_code_str, err
                    );
                    SGRCode::Noop as usize
                })
            })
            .map(|sgr_code_num| {
                num::FromPrimitive::from_usize(sgr_code_num).unwrap_or_else(|| {
                    println!(
                        "Transforming unknown SGR CSI number '{}' to Noop",
                        sgr_code_num,
                    );
                    SGRCode::Noop
                })
            })
            .collect();
        Some(EscapeSequence::SelectGraphicRendition(params))
    }

    pub fn new() -> EscapeSequenceParser {
        EscapeSequenceParser {
            sequence_type: SequenceType::Undetermined,
            csi_parameter_chars: vec![],
            csi_intermediate_chars: vec![],
        }
    }
}

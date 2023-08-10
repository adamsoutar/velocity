use std::num::IntErrorKind;

use crate::constants::special_characters::*;

use super::sequence::{CharacterSet, EscapeSequence, SGRCode, SetCursorPositionArgs};

#[derive(PartialEq, Debug)]
enum SequenceType {
    Undetermined,       // Don't know yet (just ESC so far)
    CSI,                // Control Sequence Introducer (ESC followed by "[")
    DCS,                // Device Control String (ESC followed by "P")
    OSC,                // Operating System Command (ESC followed by "]")
    DesignateG0Charset, // Picks a VT100 character set (ESC followed by "(")
    NonStandard,        // Special ones made up by other programmers (ESC followed by a space)
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
    parameter_chars: Vec<char>,
    intermediate_chars: Vec<char>,
}

impl EscapeSequenceParser {
    // Returns whether the sequence is over
    pub fn parse_character(&mut self, c: char) -> SequenceFinished {
        // println!("Parsing: {}", c);
        if self.sequence_type == SequenceType::Undetermined {
            if c == special_case_introducer::SCROLL_UP {
                // Special case, this sequence doesn't use an introducer
                // It's used by programs like less and more
                return SequenceFinished::Yes(Some(
                    EscapeSequence::MoveCursorUpScrollingIfNecessary,
                ));
            }

            self.sequence_type = match c {
                CONTROL_SEQUENCE_INTRODUCER => SequenceType::CSI,
                DEVICE_CONTROL_STRING => SequenceType::DCS,
                OPERATING_SYSTEM_COMMAND => SequenceType::OSC,
                DESIGNATE_G0_CHARACTER_SET => SequenceType::DesignateG0Charset,
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
            SequenceType::DesignateG0Charset => self.parse_g0_designate_charset_character(c),
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

    fn parse_g0_designate_charset_character(&mut self, c: char) -> SequenceFinished {
        self.parameter_chars.push(c);
        match c {
            // There are multi-char sets like "%2"
            '&' | '"' | '%' => return SequenceFinished::No,
            _ => {}
        }

        let param_str: String = self.parameter_chars.iter().collect();
        let char_set = match &param_str[..] {
            "B" => CharacterSet::UnitedStatesASCII,
            _ => {
                println!("Unsupported G0 character set designation '{}'", param_str);
                CharacterSet::UnitedStatesASCII
            }
        };
        SequenceFinished::Yes(Some(EscapeSequence::DesignateG0CharacterSet(char_set)))
    }

    fn parse_csi_character(&mut self, c: char) -> SequenceFinished {
        match c as usize {
            0x30..=0x3F => self.parameter_chars.push(c),
            0x20..=0x2F => self.intermediate_chars.push(c),
            0x40..=0x7E => return SequenceFinished::Yes(self.parse_csi_final_byte(c)),
            _ => {
                println!("Ignored unknown control sequence character '{}'", c);
            }
        }

        SequenceFinished::No
    }

    fn parse_csi_final_byte(&mut self, c: char) -> Option<EscapeSequence> {
        if self.parameter_chars.len() > 0 && self.parameter_chars[0] == '?' {
            // If a sequence begins ESC [ ?, it's a "private sequence". These are not standard,
            // but some of them (like "bracketed paste mode") are so commonly used that we should
            // support them.
            return self.parse_csi_private_sequence_final_byte(c);
        }

        match c {
            'A' => Some(EscapeSequence::MoveCursorUp(
                self.parse_csi_single_number_parameter(),
            )),
            'B' => Some(EscapeSequence::MoveCursorDown(
                self.parse_csi_single_number_parameter(),
            )),
            'C' => Some(EscapeSequence::MoveCursorForward(
                self.parse_csi_single_number_parameter(),
            )),
            'D' => Some(EscapeSequence::MoveCursorBack(
                self.parse_csi_single_number_parameter(),
            )),
            'E' => Some(EscapeSequence::MoveCursorToNextLine(
                self.parse_csi_single_number_parameter(),
            )),
            'F' => Some(EscapeSequence::MoveCursorToPreviousLine(
                self.parse_csi_single_number_parameter(),
            )),
            'G' => Some(EscapeSequence::MoveCursorHorizontalAbsolute(
                self.parse_csi_single_number_parameter(),
            )),
            'H' => self.parse_csi_set_cursor_position(),
            'J' => self.parse_csi_erase_in_display(),
            'K' => self.parse_csi_erase_in_line(),
            'P' => Some(EscapeSequence::DeleteCharacters(
                self.parse_csi_single_number_parameter(),
            )),
            'b' => Some(EscapeSequence::RepeatPreviousCharacter(
                self.parse_csi_single_number_parameter(),
            )),
            'h' => self.parse_csi_set_mode(),
            'l' => self.parse_csi_reset_mode(),
            'm' => self.parse_csi_select_graphic_rendition(),
            'c' => Some(EscapeSequence::FullReset),
            _ => {
                let inter_string: String = self.intermediate_chars.iter().collect();
                let param_string: String = self.parameter_chars.iter().collect();
                println!(
                    "Ignoring CSI '[{}{}{}' due to unknown final byte",
                    inter_string, param_string, c
                );
                None
            }
        }
    }

    fn parse_csi_single_number_parameter(&mut self) -> isize {
        let param_string: String = self.parameter_chars.iter().collect();
        param_string.parse::<isize>().unwrap_or_else(|err| {
            // Let's not be noisy about omitted args.
            if *err.kind() != IntErrorKind::Empty {
                println!(
                    "Error parsing CSI single number parameter '{}', defaulting to 1\n{:?}",
                    param_string, err
                );
            }
            1
        })
    }

    fn parse_csi_set_cursor_position(&mut self) -> Option<EscapeSequence> {
        let mut x = 1;
        let mut y = 1;

        if self.parameter_chars.len() > 0 {
            let param_string: String = self.parameter_chars.clone().into_iter().collect();
            let split: Vec<&str> = param_string.split(';').collect();
            // There's at least a y argument because param chars > 0
            // NOTE: It's "row", then "column", not x then y
            y = split[0].parse::<usize>().unwrap_or_else(|err| {
                println!(
                    "Error parsing SetCursorPosition x arg '{}', defaulting to 1\n{:?}",
                    split[0], err
                );
                1
            });
            // If there's still more to go, they're supplying x as well
            if split.len() > 1 {
                x = split[1].parse::<usize>().unwrap_or_else(|err| {
                    println!(
                        "Error parsing SetCursorPosition y arg '{}', defaulting to 1\n{:?}",
                        split[0], err
                    );
                    1
                });
            }
        }

        Some(EscapeSequence::SetCursorPosition(SetCursorPositionArgs {
            x,
            y,
        }))
    }

    fn parse_csi_private_sequence_final_byte(&mut self, c: char) -> Option<EscapeSequence> {
        let p_str: String = self.parameter_chars.iter().collect();
        match c {
            // Eg. Enable bracketed paste is ESC[?2004h
            _ if p_str == "?2004" && c == 'h' => {
                Some(EscapeSequence::PrivateEnableBracketedPasteMode)
            }
            _ if p_str == "?2004" && c == 'l' => {
                Some(EscapeSequence::PrivateDisableBracketedPasteMode)
            }
            _ if p_str == "?7" && c == 'h' => Some(EscapeSequence::EnableAutoWrapMode),
            _ if p_str == "?7" && c == 'l' => Some(EscapeSequence::DisableAutoWrapMode),
            _ if p_str == "?1" && c == 'h' => Some(EscapeSequence::SwitchToApplicationCursorKeys),
            _ if p_str == "?1" && c == 'l' => Some(EscapeSequence::SwitchToNormalCursorKeys),
            _ if p_str == "?25" && c == 'h' => Some(EscapeSequence::ShowCursor),
            _ if p_str == "?25" && c == 'l' => Some(EscapeSequence::HideCursor),
            _ => {
                println!("Ignoring unknown CSI private sequence '{}', '{}'", p_str, c);
                None
            }
        }
    }

    fn parse_csi_set_mode(&mut self) -> Option<EscapeSequence> {
        let mode_type = self.parse_csi_set_or_reset_mode_parameter();
        let maybe_set_mode_enum = num::FromPrimitive::from_usize(mode_type);
        if maybe_set_mode_enum.is_none() {
            println!("Unknown CSI set mode type '{}'", mode_type);
            return None;
        } else {
            return Some(EscapeSequence::SetMode(maybe_set_mode_enum.unwrap()));
        }
    }

    fn parse_csi_reset_mode(&mut self) -> Option<EscapeSequence> {
        let mode_type = self.parse_csi_set_or_reset_mode_parameter();
        let maybe_set_mode_enum = num::FromPrimitive::from_usize(mode_type);
        if maybe_set_mode_enum.is_none() {
            println!("Unknown CSI reset mode type '{}'", mode_type);
            return None;
        } else {
            return Some(EscapeSequence::ResetMode(maybe_set_mode_enum.unwrap()));
        }
    }

    fn parse_csi_erase_in_line(&mut self) -> Option<EscapeSequence> {
        let erase_type = self.parse_csi_erase_type_number();
        let maybe_erase_type_enum = num::FromPrimitive::from_usize(erase_type);
        if maybe_erase_type_enum.is_none() {
            println!("Unknown CSI line erase type '{}'", erase_type);
            return None;
        } else {
            return Some(EscapeSequence::EraseInLine(maybe_erase_type_enum.unwrap()));
        }
    }

    fn parse_csi_erase_in_display(&mut self) -> Option<EscapeSequence> {
        let erase_type = self.parse_csi_erase_type_number();
        let maybe_erase_type_enum = num::FromPrimitive::from_usize(erase_type);
        if maybe_erase_type_enum.is_none() {
            println!("Unknown CSI display erase type '{}'", erase_type);
            return None;
        } else {
            return Some(EscapeSequence::EraseInDisplay(
                maybe_erase_type_enum.unwrap(),
            ));
        }
    }

    fn parse_csi_erase_type_number(&mut self) -> usize {
        // This is the default value if there are no parameter bytes
        let mut erase_type = 0;

        if self.parameter_chars.len() == 1 {
            erase_type = self.parameter_chars[0]
                .to_string()
                .parse::<usize>()
                .unwrap_or_else(|err| {
                    println!(
                        "Error parsing CSI erase type '{}', returning Noop\n{:?}",
                        self.parameter_chars[0], err
                    );
                    999
                });
        }

        erase_type
    }

    fn parse_csi_set_or_reset_mode_parameter(&mut self) -> usize {
        let param_string: String = self.parameter_chars.iter().collect();
        let mode_param = param_string.parse::<usize>().unwrap_or_else(|err| {
            println!(
                "Error parsing CSI Set/Reset mode type '{}', returning AutomaticNewline\n{:?}",
                param_string, err
            );
            20 // (AutomaticNewline)
        });

        mode_param
    }

    fn parse_csi_select_graphic_rendition(&mut self) -> Option<EscapeSequence> {
        if self.parameter_chars.len() == 0 {
            // No arguments is a reset (top sends this)
            return Some(EscapeSequence::SelectGraphicRendition(vec![
                SGRCode::ResetAllTextStyles,
            ]));
        }

        let param_string: String = self.parameter_chars.iter().collect();
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
            parameter_chars: vec![],
            intermediate_chars: vec![],
        }
    }
}

use crate::constants::{special_characters::*, *};
use crate::shell_layer::{get_shell_layer, ShellLayer};

pub struct TtySize {
    pub cols: usize,
    pub rows: usize,
}

pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
}

// Not vim-related.
// Confession hidden deep in my source code: I have no idea how to use vim.
// Indicates whether we're mid escape-sequence parsing or not
enum InsertionMode {
    Standard,
    EscapeSequence,
}

pub struct TtyState {
    pub size: TtySize,
    pub cursor_pos: CursorPosition,
    pub scrollback_start: usize,
    pub scrollback_buffer: Vec<Vec<char>>,
    read_buffer: [u8; FD_BUFFER_SIZE_BYTES],
    read_buffer_length: usize,
    shell_layer: Box<dyn ShellLayer>,
    // As we encounter unfinished unicode surrogates, we'll push them here.
    // When we find the end, the bytes will encode a char.
    current_unicode_scalar: Vec<u8>,
    // A unicode scalar is up to four bytes long, and we'll know how long it's
    // going to be when it starts
    remaining_unicode_scalar_bytes: u8,
    insertion_mode: InsertionMode,
}

impl TtyState {
    // Returns a character *if* it's finished
    fn parse_partial_unicode(&mut self, b: u8) -> Option<char> {
        if self.remaining_unicode_scalar_bytes == 0 {
            // This is the start of a new character
            if b & 0b1000_0000 == 0 {
                // It's a standalone character. Nice and simple!
                return Some(b as char);
            }

            // Else it indicates to us how long it's *going* to be
            self.remaining_unicode_scalar_bytes = match b {
                b if b & 0b1110_0000 == 0b1100_0000 => 2,
                b if b & 0b1111_0000 == 0b1110_0000 => 3,
                b if b & 0b1111_1000 == 0b1111_0000 => 4,
                // Invalid character start, we've broken
                _ => return Some(REPLACEMENT_CHARACTER),
            };
        }

        // NOTE: At this point, if b doesn't begin with 0b10, it's actually invalid,
        //   but we'll be loose and wait for the stdlib func to catch it.
        self.current_unicode_scalar.push(b);
        self.remaining_unicode_scalar_bytes -= 1;

        if self.remaining_unicode_scalar_bytes == 0 {
            // This character is finished
            // This StdLib function will automagically return UNICODE_REPLACEMENT_CHARACTER if
            // what we pass it isn't valid UTF-8
            let parsed_chars: Vec<char> = String::from_utf8_lossy(&self.current_unicode_scalar)
                .chars()
                .collect();
            return Some(parsed_chars[0]);
        }

        return None;
    }

    fn insert_byte(&mut self, next_byte: u8) {
        if let Some(parsed_char) = self.parse_partial_unicode(next_byte) {
            match self.insertion_mode {
                InsertionMode::Standard => self.standard_insert_char(parsed_char),
                InsertionMode::EscapeSequence => self.escape_insert_char(parsed_char),
            }
        }
    }

    fn escape_insert_char(&mut self, c: char) {
        match c {
            // Most control sequences end with a lowercase letter,
            // so this is probably a fine-ish way to ignore codes for now
            'a' => {}
            'b' => {}
            'c' => {}
            'd' => {}
            'e' => {}
            'f' => {}
            'g' => {}
            'h' => {}
            'i' => {}
            'j' => {}
            'k' => {}
            'l' => {}
            'm' => {}
            'n' => {}
            'o' => {}
            'p' => {}
            'q' => {}
            'r' => {}
            's' => {}
            't' => {}
            'u' => {}
            'v' => {}
            'w' => {}
            'x' => {}
            'y' => {}
            'z' => {}
            _ => return,
        }
        self.insertion_mode = InsertionMode::Standard
    }

    fn standard_insert_char(&mut self, c: char) {
        if c == ESCAPE {
            // That's the start of an escape sequence
            self.insertion_mode = InsertionMode::EscapeSequence;
            return;
        }

        let cursor_line = self.scrollback_start + self.cursor_pos.y;
        while self.scrollback_buffer.len() <= cursor_line {
            self.scrollback_buffer.push(vec![]);
        }

        let mut line_buffer = &mut self.scrollback_buffer[cursor_line];
        if c == NEWLINE || self.cursor_pos.x >= 80 {
            self.cursor_pos.x = 0;
            self.cursor_pos.y += 1;

            // If we're pushed too low, scroll
            if self.cursor_pos.y >= self.size.rows {
                self.cursor_pos.y -= 1;
                self.scrollback_start += 1;
            }

            self.scrollback_buffer.push(vec![]);
            line_buffer = &mut self.scrollback_buffer[self.scrollback_start + self.cursor_pos.y];
            if c == '\n' {
                return;
            }
        }

        if c == BACKSPACE {
            line_buffer.pop();
            self.cursor_pos.x -= 1;
        } else {
            line_buffer.insert(self.cursor_pos.x, c);
            self.cursor_pos.x += 1;
        }
    }

    pub fn read(&mut self) {
        self.shell_layer
            .read(&mut self.read_buffer, &mut self.read_buffer_length);

        if self.read_buffer_length == 0 {
            return;
        }

        for i in 0..self.read_buffer_length {
            self.insert_byte(self.read_buffer[i])
        }
    }

    pub fn write(&mut self, data: &[u8]) {
        self.shell_layer.write(data);
    }

    pub fn new() -> Self {
        let size = TtySize { cols: 80, rows: 25 };
        let shell_layer = get_shell_layer(size.rows, size.cols);
        TtyState {
            size,
            cursor_pos: CursorPosition { x: 0, y: 0 },
            scrollback_start: 0,
            scrollback_buffer: vec![],
            read_buffer: [0; FD_BUFFER_SIZE_BYTES],
            read_buffer_length: 0,
            shell_layer,
            current_unicode_scalar: vec![],
            remaining_unicode_scalar_bytes: 0,
            insertion_mode: InsertionMode::Standard,
        }
    }
}

use crate::constants::*;
use crate::shell_layer::{get_shell_layer, ShellLayer};

pub struct TtySize {
    pub cols: usize,
    pub rows: usize,
}

pub struct CursorPosition {
    pub x: usize,
    pub y: usize,
}

pub struct TtyState {
    pub size: TtySize,
    pub cursor_pos: CursorPosition,
    pub scrollback_start: usize,
    pub scrollback_buffer: Vec<Vec<char>>,
    read_buffer: [u8; FD_BUFFER_SIZE_BYTES],
    read_buffer_length: usize,
    shell_layer: Box<dyn ShellLayer>,
}

impl TtyState {
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
        }
    }

    pub fn read(&mut self) {
        self.shell_layer
            .read(&mut self.read_buffer, &mut self.read_buffer_length);

        if self.read_buffer_length == 0 {
            return;
        }

        println!(
            "Cursor pos: x: {}, y: {}",
            self.cursor_pos.x, self.cursor_pos.y
        );

        // We have new characters, let's put them at the cursor location
        let cursor_line = self.scrollback_start + self.cursor_pos.y;
        println!("cursor_line: {}", cursor_line);
        while self.scrollback_buffer.len() <= cursor_line {
            self.scrollback_buffer.push(vec![]);
        }
        // TODO: If that pushed things so that scrollback_start is more than 25 lines from the end,
        //   scroll down.

        let mut line_buffer = &mut self.scrollback_buffer[cursor_line];
        for i in 0..self.read_buffer_length {
            // TODO: Handle UTF-8 pairs
            let c = self.read_buffer[i] as char;
            if c == '\n' || self.cursor_pos.x >= 80 {
                self.cursor_pos.x = 0;
                self.cursor_pos.y += 1;
                self.scrollback_buffer.push(vec![]);
                line_buffer =
                    &mut self.scrollback_buffer[self.scrollback_start + self.cursor_pos.y];
                continue;
            }
            line_buffer.insert(self.cursor_pos.x, c);
            self.cursor_pos.x += 1;
        }
    }
}

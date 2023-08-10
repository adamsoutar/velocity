use std::cmp::{max, min};
use std::collections::VecDeque;

use crate::constants::{special_characters::*, *};
use crate::escape_sequence::parser::{EscapeSequenceParser, SequenceFinished};
use crate::escape_sequence::sequence::{
    CharacterSet, EraseInDisplayType, EraseInLineType, EscapeSequence, SetCursorPositionArgs,
    SetOrResetModeType,
};
use crate::shell_layer::{get_shell_layer, ShellLayer};
use crate::text_styles::decorated_char::DecoratedChar;
use crate::text_styles::text_style::TextStyle;

pub struct TtySize {
    pub cols: usize,
    pub rows: usize,
}

pub struct CursorPosition {
    pub x: isize,
    pub y: isize,
}

// Not vim-related.
// Confession hidden deep in my source code: I have no idea how to use vim.
enum InsertionMode {
    // Also known as AutomaticNewline or LN.
    // Chars are put into the cursor position and replace any already there.
    // When the cursor reaches the right edge of the line buffer, it wraps.
    Replace,
    // Also known as InsertionReplacement or IRM.
    // Chars are inserted into the line buffer at the cursor index and push others to the right.
    // Any chars that are pushed off the right edge of the line buffer are lost.
    Insert,
}

type LineType = VecDeque<DecoratedChar>;
type ScrollbackBufferType = VecDeque<LineType>;

pub struct TtyState {
    pub size: TtySize,
    pub cursor_pos: CursorPosition,
    pub cursor_visible: bool,
    pub scrollback_start: usize,
    pub scrollback_buffer: ScrollbackBufferType,
    pub bracketed_paste_mode: bool,
    character_set: CharacterSet,
    read_buffer: [u8; FD_BUFFER_SIZE_BYTES],
    read_buffer_length: usize,
    shell_layer: Box<dyn ShellLayer>,
    // As we encounter unfinished unicode surrogates, we'll push them here.
    // When we find the end, the bytes will encode a char.
    current_unicode_scalar: Vec<u8>,
    // A unicode scalar is up to four bytes long, and we'll know how long it's
    // going to be when it starts
    remaining_unicode_scalar_bytes: u8,
    previously_inserted_character: char,
    insertion_mode: InsertionMode,
    escape_sequence_parser: EscapeSequenceParser,
    am_parsing_escape_sequence: bool,
    text_style: TextStyle,
    // This is a strange feature where we wait when we get to the edge of the screen
    // for one character before doing a newline. This is so that programs can print
    // exactly 80 characters on an 80 character screen without breaking, but 81 chars
    // breaks the line.
    // The name is taken from SerenityOS' LibVT. I can't find a standard name for it.
    stomp: bool,
    // (Private xterm extension) when this is enabled, the cursor wraps when it hits
    // (and stomps at) the end of the screen. Programs can disable it with a private
    //  escape code.
    autowrap: bool,
    // Cursor keys are either 'normal' or 'application'. This changes the byte we
    // send in the middle of cursor key inputs. Similar to bracketed paste, see
    // frontends like SFML for actual implementation.
    pub application_cursor_keys: bool,
}

impl TtyState {
    fn apply_escape_sequence(&mut self, seq: &EscapeSequence) {
        match seq {
            EscapeSequence::MoveCursorUp(n) => {
                self.set_cursor_pos(self.cursor_pos.x, self.cursor_pos.y - n)
            }
            EscapeSequence::MoveCursorDown(n) => {
                self.set_cursor_pos(self.cursor_pos.x, self.cursor_pos.y + n)
            }
            // TODO: Does this break over line boundaries?
            EscapeSequence::MoveCursorForward(n) => {
                self.set_cursor_pos(self.cursor_pos.x + n, self.cursor_pos.y)
            }
            EscapeSequence::MoveCursorBack(n) => {
                self.set_cursor_pos(self.cursor_pos.x - n, self.cursor_pos.y)
            }
            EscapeSequence::MoveCursorToNextLine(n) => {
                self.set_cursor_pos(0, self.cursor_pos.y + n)
            }
            EscapeSequence::MoveCursorToPreviousLine(n) => {
                self.set_cursor_pos(0, self.cursor_pos.y - n)
            }
            EscapeSequence::MoveCursorHorizontalAbsolute(n) => {
                self.set_cursor_pos(*n, self.cursor_pos.y)
            }
            EscapeSequence::SetCursorPosition(p) => self.apply_sequence_set_cursor_position(p),
            EscapeSequence::EraseInDisplay(e) => self.apply_sequence_erase_in_display(e),
            EscapeSequence::EraseInLine(e) => self.apply_sequence_erase_in_line(e),
            EscapeSequence::SelectGraphicRendition(_) => self.text_style.apply_escape_sequence(seq),
            EscapeSequence::PrivateEnableBracketedPasteMode => self.bracketed_paste_mode = true,
            EscapeSequence::PrivateDisableBracketedPasteMode => self.bracketed_paste_mode = false,
            EscapeSequence::EnableAutoWrapMode => self.autowrap = true,
            EscapeSequence::DisableAutoWrapMode => self.autowrap = false,
            EscapeSequence::FullReset => self.apply_sequence_full_reset(),
            EscapeSequence::SwitchToApplicationCursorKeys => self.application_cursor_keys = true,
            EscapeSequence::SwitchToNormalCursorKeys => self.application_cursor_keys = false,
            EscapeSequence::MoveCursorUpScrollingIfNecessary => {
                self.apply_sequence_move_cursor_up_scrolling_if_necessary()
            }
            EscapeSequence::SetMode(m) => self.apply_sequence_set_mode(m),
            EscapeSequence::ResetMode(m) => self.apply_sequence_reset_mode(m),
            EscapeSequence::DeleteCharacters(n) => self.apply_sequence_delete_characters(*n),
            EscapeSequence::ShowCursor => self.cursor_visible = true,
            EscapeSequence::HideCursor => self.cursor_visible = false,
            EscapeSequence::DesignateG0CharacterSet(cs) => self.character_set = *cs,
            EscapeSequence::RepeatPreviousCharacter(n) => {
                self.apply_sequence_repeat_previous_character(*n)
            }
            // As we go through the process of implementing these, we'll keep adding new
            // parsing code that then makes this match arm reachable.
            #[allow(unreachable_patterns)]
            _ => println!("Unhandled escape sequence {:?}", seq),
        }
    }

    fn apply_sequence_repeat_previous_character(&mut self, count: isize) {
        for _ in 0..count {
            self.standard_insert_char(self.previously_inserted_character);
        }
    }

    fn apply_sequence_delete_characters(&mut self, count: isize) {
        if count < 0 {
            println!("Asked to DeleteCharacters with N < 0. Ignoring");
            return;
        }

        let old_x = self.cursor_pos.x;
        let line_buffer = self.get_current_line_ref();
        for i in 0..count {
            line_buffer.remove((old_x - i) as usize);
        }
        self.cursor_pos.x -= count;
    }

    fn apply_sequence_set_mode(&mut self, mode: &SetOrResetModeType) {
        self.insertion_mode = match mode {
            // xterm/hterm have different names for these, compared to what I like to call them.
            // I get confused by their names. Luckily, this here is my program, and I can do
            // what I want. And I get to set my own bed time. Take that, mum!
            SetOrResetModeType::AutomaticNewline => InsertionMode::Replace,
            SetOrResetModeType::InsertionReplacement => InsertionMode::Insert,
        };
    }

    fn apply_sequence_reset_mode(&mut self, _mode: &SetOrResetModeType) {
        // This can do more, especially if you support mode insertion modes, but Velocity
        // only supports this system.
        self.insertion_mode = InsertionMode::Replace;
    }

    fn apply_sequence_full_reset(&mut self) {
        // Here we don't reset things like unicode parser state.
        // Those are too low-level, we're only emulating a VT-100 reset.
        self.cursor_pos = CursorPosition { x: 0, y: 0 };
        self.scrollback_start = 0;
        self.scrollback_buffer = VecDeque::with_capacity(self.size.rows);
        self.bracketed_paste_mode = false;
        self.text_style = TextStyle::new();
        self.stomp = false;
        self.autowrap = false;
    }

    fn apply_sequence_move_cursor_up_scrolling_if_necessary(&mut self) {
        if self.cursor_pos.y == 0 {
            if self.scrollback_start == 0 {
                return;
            }

            self.scrollback_start -= 1;
            self.scrollback_buffer.pop_back();
        } else {
            self.set_cursor_pos(self.cursor_pos.x, self.cursor_pos.y - 1);
        }
    }

    fn apply_sequence_set_cursor_position(&mut self, args: &SetCursorPositionArgs) {
        // These args are 1-indexed, but our cursor is 0-indexed.
        self.set_cursor_pos(args.x as isize - 1, args.y as isize - 1)
    }

    fn set_cursor_pos(&mut self, x: isize, y: isize) {
        let max_x = self.size.cols as isize - 1;
        let max_y = self.size.rows as isize - 1;
        self.cursor_pos.x = min(max(x, 0), max_x);
        self.cursor_pos.y = min(max(y, 0), max_y);
        self.stomp = false;
    }

    fn apply_sequence_erase_in_display(&mut self, erase_type: &EraseInDisplayType) {
        if *erase_type == EraseInDisplayType::ToEndOfScreen
            || *erase_type == EraseInDisplayType::EntireScreen
        {
            // First, erase the rest of the current line
            self.apply_sequence_erase_in_line(&EraseInLineType::ToEndOfLine);

            // Then, erase all following lines
            let start_line = self.scrollback_start + self.cursor_pos.y as usize + 1;
            for _ in start_line..self.scrollback_buffer.len() {
                self.scrollback_buffer.pop_front();
            }
        }

        if *erase_type == EraseInDisplayType::ToStartOfScreen
            || *erase_type == EraseInDisplayType::EntireScreen
        {
            // First, erase the start of the current line
            self.apply_sequence_erase_in_line(&EraseInLineType::ToStartOfLine);

            // Then, erase all previous lines
            for i in 0..self.scrollback_start + self.cursor_pos.y as usize {
                self.scrollback_buffer[i].clear();
            }
        }

        // TODO: iTerm has some kind of permissions system with an option to diasallow
        //   programs from doing this. We should offer the same.
        if *erase_type == EraseInDisplayType::EntireScreenAndScrollbackBuffer {
            self.scrollback_buffer.clear();
            self.scrollback_start = 0;
        }
    }

    fn apply_sequence_erase_in_line(&mut self, erase_type: &EraseInLineType) {
        let cursor_x = self.cursor_pos.x;
        let line = self.get_current_line_ref();

        if *erase_type == EraseInLineType::ToEndOfLine || *erase_type == EraseInLineType::EntireLine
        {
            let diff = line.len() as isize - cursor_x;
            if diff > 0 {
                // Truncate takes the amount of elements you want to keep from the left, NOT the
                // number to cut off the right.
                line.truncate(line.len() - diff as usize)
            }
        }

        if *erase_type == EraseInLineType::ToStartOfLine
            || *erase_type == EraseInLineType::EntireLine
        {
            let diff = cursor_x - line.len() as isize;
            if diff > 0 {
                println!(
                    "We've been asked to EraseToStartOfLine at cursor {}, line length: {}",
                    cursor_x,
                    line.len()
                );
                // This is just an efficient way to truncate() the other side.
                // VecDeque doesn't have truncate_front
                drop(line.drain(0..cursor_x as usize))
            }
        }
    }

    fn ensure_backing_store_for_current_line(&mut self) {
        let cursor_line = self.scrollback_start + self.cursor_pos.y as usize;
        while self.scrollback_buffer.len() <= cursor_line {
            self.scrollback_buffer
                .push_back(VecDeque::with_capacity(self.size.cols));
        }
    }

    fn get_current_line_ref(&mut self) -> &mut LineType {
        self.ensure_backing_store_for_current_line();
        &mut self.scrollback_buffer[self.scrollback_start + self.cursor_pos.y as usize]
    }

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
                _ => {
                    println!("Invalid unicode byte: {}", b);
                    return Some(REPLACEMENT_CHARACTER);
                }
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
        // TODO: There's a very light chance that someone would want a control sequence that
        //   operates purely on bytes before Unicode parsing. But most of the classic ones
        //   pre-date UTF-8 and therefore are defined in terms of ASCII.
        if let Some(parsed_char) = self.parse_partial_unicode(next_byte) {
            // println!("Char: {:?} ({})", parsed_char, parsed_char as usize);
            if self.am_parsing_escape_sequence {
                self.escape_insert_char(parsed_char);
            } else {
                self.standard_insert_char(parsed_char);
            }
        }
    }

    fn escape_insert_char(&mut self, c: char) {
        self.am_parsing_escape_sequence = match self.escape_sequence_parser.parse_character(c) {
            // This sequence isn't over yet
            SequenceFinished::No => true,
            // It's done, back to normals
            SequenceFinished::Yes(parse_result) => {
                println!("Parsed: {:?}", parse_result);
                if let Some(sequence) = parse_result {
                    self.apply_escape_sequence(&sequence);
                }
                false
            }
        }
    }

    fn standard_insert_char(&mut self, c: char) {
        // TODO: Also support the unicode shortcuts like the CSI character.
        //   Just spawn a parser and immediately pretend you saw the next char.
        if c == ESCAPE {
            // That's the start of an escape sequence
            self.escape_sequence_parser = EscapeSequenceParser::new();
            self.am_parsing_escape_sequence = true;
            return;
        }

        self.ensure_backing_store_for_current_line();
        self.previously_inserted_character = c;

        match c {
            BACKSPACE | CARRIAGE_RETURN | HORIZONTAL_TAB | BELL | FORMFEED => {
                self.handle_c0_control_code(c);
                return;
            }
            _ => {}
        }

        // From now on, we know it's a printable character. So we need to handle things like spacing
        // and wrapping
        let cursor_line = self.scrollback_start + self.cursor_pos.y as usize;
        let mut line_buffer = &mut self.scrollback_buffer[cursor_line];

        let cursor_needs_to_wrap = self.cursor_pos.x as usize >= self.size.cols - 1 && self.stomp;
        if c == NEWLINE || (cursor_needs_to_wrap && self.autowrap) {
            self.stomp = false;
            self.cursor_pos.x = 0;
            self.cursor_pos.y += 1;

            // If we're pushed too low, scroll
            if self.cursor_pos.y as usize >= self.size.rows {
                self.cursor_pos.y -= 1;
                self.scrollback_start += 1;
            }

            while self.scrollback_start + self.cursor_pos.y as usize >= self.scrollback_buffer.len()
            {
                self.scrollback_buffer
                    .push_back(VecDeque::with_capacity(self.size.cols));
            }

            line_buffer =
                &mut self.scrollback_buffer[self.scrollback_start + self.cursor_pos.y as usize];
            if c == NEWLINE {
                return;
            }
        }

        let d_c = DecoratedChar::new(c, self.text_style);
        while line_buffer.len() <= self.cursor_pos.x as usize {
            line_buffer.push_back(DecoratedChar::new(' ', self.text_style));
        }
        match self.insertion_mode {
            InsertionMode::Replace => line_buffer[self.cursor_pos.x as usize] = d_c,
            InsertionMode::Insert => {
                line_buffer.insert(self.cursor_pos.x as usize, d_c);
                while line_buffer.len() > self.size.cols {
                    line_buffer.pop_back();
                }
            }
        }

        if self.cursor_pos.x as usize == self.size.cols - 1 {
            // Slightly unusual legacy behaviour. See the comment in the TtyState struct
            self.stomp = true;
        } else {
            self.cursor_pos.x += 1;
        }
    }

    fn handle_c0_control_code(&mut self, c: char) {
        match c {
            BACKSPACE => {
                // TODO: Is backspace treated differently during InsertionMode::Insert?
                self.cursor_pos.x -= 1
            }
            CARRIAGE_RETURN => self.cursor_pos.x = 0,
            HORIZONTAL_TAB => {
                // Move the cursor right to the next multiple of 8
                self.cursor_pos.x += 8 - (self.cursor_pos.x % 8);
            }
            _ => println!("Unimplemented c0 control code {:?} ({})", c, c as usize),
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

    pub fn new(cols: usize, rows: usize) -> Self {
        let size = TtySize { cols, rows };
        let shell_layer = get_shell_layer(size.rows, size.cols);
        TtyState {
            size,
            cursor_pos: CursorPosition { x: 0, y: 0 },
            cursor_visible: true,
            scrollback_start: 0,
            scrollback_buffer: VecDeque::with_capacity(rows),
            bracketed_paste_mode: false,
            character_set: CharacterSet::UnitedStatesASCII,
            read_buffer: [0; FD_BUFFER_SIZE_BYTES],
            read_buffer_length: 0,
            shell_layer,
            current_unicode_scalar: vec![],
            remaining_unicode_scalar_bytes: 0,
            previously_inserted_character: ' ',
            insertion_mode: InsertionMode::Replace,
            escape_sequence_parser: EscapeSequenceParser::new(),
            am_parsing_escape_sequence: false,
            text_style: TextStyle::new(),
            stomp: false,
            autowrap: true,
            application_cursor_keys: false,
        }
    }
}

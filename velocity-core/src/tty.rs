pub struct CursorPosition {
    pub x: u8,
    pub y: u8,
}

pub struct TtyState {
    pub cursor_pos: CursorPosition,
    pub scrollback_start: usize,
}

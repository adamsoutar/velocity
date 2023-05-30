use std::ffi::c_int;

// Eg. on a ProMotion machine, change this to 120
pub const TARGET_FRAMERATE: usize = 60;
// This is subtracted from our poll allowance so we don't spend our entire frame budget on polling
pub const RENDER_ALLOWANCE_MS: c_int = 3;
// The maximum amount of time per-frame that we will wait for new bytes from the shell program.
// This is so that we actually end up drawing and responding to user input instead of waiting
// forever on a stalled program.
pub const FD_POLL_TIMEOUT_MS: c_int = 1000 / TARGET_FRAMERATE as c_int - RENDER_ALLOWANCE_MS;
// The amount of data we'll ask the child program file descriptor for at a time.
// Essentially how many character chunks we're confident we can draw in one go without
// stalling.
pub const FD_BUFFER_SIZE_BYTES: usize = 4096;

pub mod special_characters {
    // Question mark in a black diamond
    pub const REPLACEMENT_CHARACTER: char = '\u{FFFD}';
    pub const BACKSPACE: char = '\u{0008}';
    // Introduces escape sequences
    pub const ESCAPE: char = '\u{001B}';
    // Causes a ding noise when printed. Well... it should do
    pub const BELL: char = '\u{0007}';
    pub const HORIZONTAL_TAB: char = '\u{0009}';
    pub const NEWLINE: char = '\u{000A}'; // \n
    pub const VERTICAL_TAB: char = '\u{000B}';
    // I think this does new page - like "cls" on Windows
    pub const FORMFEED: char = '\u{000C}';
    // Pushes to the start of the line like a typewriter. Used for progress bars
    pub const CARRIAGE_RETURN: char = '\u{000D}';
    pub const DELETE: char = '\u{007F}';
}

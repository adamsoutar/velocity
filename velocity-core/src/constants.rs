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
// This is the question mark in a black diamond we'll print when our unicode parser falls over.
pub const UNICODE_REPLACEMENT_CHARACTER: char = '\u{FFFD}';

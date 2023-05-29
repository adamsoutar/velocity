use crate::constants::*;

mod mac_os;

// This represents an abstraction of getting bytes to and from the shell program,
// for each different platform (eg macOS vs Linux).
pub trait ShellLayer {
    // This callback is called when we get bytes from the child process, OR if it has been long
    // enough since we last rendered a frame (a timeout is hit)
    fn execute_and_run_shell(&mut self, callback: fn(&[u8; FD_BUFFER_SIZE_BYTES], usize));
    // This is called by our GUI layer when the user hits a keyboard key
    fn write(&mut self, data: &[u8]);
}

pub fn get_shell_layer_for_current_platform() -> Box<dyn ShellLayer> {
    // TODO: Actually check build-time flags if we ever support more than just macOS some day.
    Box::new(mac_os::MacOsShellLayer::new())
}

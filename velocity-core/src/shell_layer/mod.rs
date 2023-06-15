use crate::constants::*;

mod linux;
mod mac_os;

// This represents an abstraction of getting bytes to and from the shell program,
// for each different platform (eg macOS vs Linux).
pub trait ShellLayer {
    // This callback is called when we get bytes from the child process, OR if it has been long
    // enough since we last rendered a frame (a timeout is hit)
    fn read(&mut self, buffer: &mut [u8; FD_BUFFER_SIZE_BYTES], written: &mut usize);
    // This is called by our GUI layer when the user hits a keyboard key
    fn write(&mut self, data: &[u8]);
}

pub fn get_shell_layer(rows: usize, cols: usize) -> Box<dyn ShellLayer> {
    // TODO: Actually check build-time flags if we ever support more than just macOS some day.
    // Box::new(mac_os::MacOsShellLayer::new(rows, cols))
    Box::new(linux::LinuxShellLayer::new(rows, cols))
}

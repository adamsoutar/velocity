use crate::constants::*;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod mac_os;

// This represents an abstraction of getting bytes to and from the shell program,
// for each different platform (eg macOS vs Linux).
pub trait ShellLayer {
    // This callback is called when we get bytes from the child process, OR if it has been long
    // enough since we last rendered a frame (a timeout is hit)
    fn read(&mut self, buffer: &mut [u8; FD_BUFFER_SIZE_BYTES], written: &mut usize);
    // This is called by our GUI layer when the user hits a keyboard key
    fn write(&mut self, data: &[u8]);
    // This is called when the GUI window is resized. It's called with the number of rows and
    // collumns that fit into the new width and height of the window.
    fn resized(&mut self, new_rows: usize, new_cols: usize);
}

pub fn get_shell_layer(rows: usize, cols: usize) -> Box<dyn ShellLayer> {
    #[cfg(target_os = "macos")]
    return Box::new(mac_os::MacOsShellLayer::new(rows, cols));
    #[cfg(target_os = "linux")]
    return Box::new(linux::LinuxShellLayer::new(rows, cols));
}

use super::ShellLayer;
use crate::constants::*;

use libc::{c_void, execle, winsize};
use nix::{
    poll::{poll, PollFd, PollFlags},
    pty::{openpty, OpenptyResult},
    unistd::{close, dup2, fork, setsid, ForkResult},
};
use std::{
    ffi::CString,
    fs::File,
    io::{Read, Write},
    os::fd::FromRawFd,
    ptr,
};

mod ioctl {
    // This macro generates a function definition - it doesn't actually *do* the ioctl
    ioctl_none_bad!(tiocsctty, libc::TIOCSCTTY);
}

pub struct MacOsShellLayer {
    pty_result: Option<OpenptyResult>,
    byte_buffer: [u8; FD_BUFFER_SIZE_BYTES],
}

impl ShellLayer for MacOsShellLayer {
    fn execute_and_run_shell(&mut self, callback: fn(&[u8; FD_BUFFER_SIZE_BYTES], usize)) {
        let winsize = winsize {
            // Default size from iTerm2
            ws_col: 80,
            ws_row: 21,
            // TODO: Is this important?
            ws_xpixel: 100,
            ws_ypixel: 100,
        };
        // NOTE: This result does NOT implement Drop, so the file descriptors must be
        //   closed manually (or realistically, we will leak them).
        self.pty_result = Some(openpty(&winsize, None).expect("openpty() failed"));

        unsafe { self.fork_and_become_shell_as_child_process() }
        // From now on we're in the parent process. The child uses exec to become the shell and
        // never reaches this point.

        // Read chunks from the child process forever
        let poll_fd = PollFd::new(self.pty_result.unwrap().master, PollFlags::POLLIN);
        let mut file = unsafe { File::from_raw_fd(self.pty_result.unwrap().master) };
        let mut fd_drained = true;

        loop {
            if fd_drained {
                // Poll until the fd is ready to read again
                // This allows us to keep drawing frames and maintain a responsive
                // window even though reading the file is blocked.
                let poll_result = poll(&mut [poll_fd], FD_POLL_TIMEOUT_MS).unwrap();
                if poll_result == 0 {
                    // We timed out, now is the time to draw a frame
                    // We didn't actually read any data, so we'll send a zero
                    callback(&self.byte_buffer, 0);
                    continue;
                }
                if poll_result == -1 {
                    panic!("Poll failed");
                }
            }

            let read_count = file.read(&mut self.byte_buffer).unwrap();

            // TODO: Should we be pooling these up and sending them when the fd is drained?
            //   Currently, we'll be framerate limited when a program is printing more than 4K chars
            //   per 16ms, which is not a low threshold.
            callback(&self.byte_buffer, read_count);

            if read_count != FD_BUFFER_SIZE_BYTES {
                fd_drained = true;
            }
        }
    }

    fn write(&mut self, data: &[u8]) {
        let mut file = unsafe { File::from_raw_fd(self.pty_result.unwrap().master) };
        file.write_all(data).unwrap();
    }
}

impl MacOsShellLayer {
    pub fn new() -> Self {
        MacOsShellLayer {
            pty_result: None,
            byte_buffer: [0; FD_BUFFER_SIZE_BYTES],
        }
    }

    unsafe fn fork_and_become_shell_as_child_process(&self) {
        let fork_result = fork();

        match fork_result {
            Ok(ForkResult::Parent { .. }) => return,
            Ok(ForkResult::Child) => {}
            Err(_) => panic!("Process failed to fork"),
        }

        // The child process doesn't need a reference to the master PTY file.
        close(self.pty_result.unwrap().master).unwrap();

        // Put the child process in a different process group to the parent.
        // This is required for the shell program to spawn sub-processes and keep track of the
        // foreground process for ^C, ^Z etc.
        setsid().unwrap();

        // Set the slave file to the "controlling terminal" of this process.
        // Slightly black-box magic to me.
        let pty_slave = self.pty_result.unwrap().slave;
        ioctl::tiocsctty(pty_slave).unwrap();

        // Set basic io file descriptors for this process to read/write from the slave file
        dup2(pty_slave, 0).unwrap(); // StdIn
        dup2(pty_slave, 1).unwrap(); // StdOut
        dup2(pty_slave, 2).unwrap(); // StdErr

        close(pty_slave).unwrap();

        // python3 -c "while True: print('Hello world')"
        let shell_path = CString::new("/usr/bin/python3").unwrap();
        // let argv = [ptr::null()];
        let test_env_var = CString::new("VELOCITY=TRUE").unwrap();
        // TODO: Set TERM and TERM_PROGRAM.
        // TODO: Check if the rest of the env from the child process is inherited.
        let env = [test_env_var.as_ptr(), ptr::null()];

        // The way the variadic arguments work here are:
        // 1 - The executable path
        // 2 through N - argv items
        // N+1 - NULL terminal for argv array
        // N+2 - Pointer to an array of environment variables
        // What it's doing is replacing our executable with the image at shell_path. So we the child
        // process *become* the shell program (maintaining the modifications we made to stdio above).
        // TODO: Panic if this is -1
        let _exec_result = execle(
            shell_path.as_ptr(),
            shell_path.clone().as_ptr(),
            ptr::null() as *const c_void,
            env.as_ptr(),
        );
    }
}

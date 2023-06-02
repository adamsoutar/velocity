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
    pty_result: OpenptyResult,
    fd_drained: bool,
    master_fd_file: Option<File>,
}

impl ShellLayer for MacOsShellLayer {
    fn read(&mut self, buffer: &mut [u8; FD_BUFFER_SIZE_BYTES], written: &mut usize) {
        // TODO: We can also poll() for POLLNVAL to check that the file descriptor is still open.
        //   We need to do this to check if the child process died (eg. shell did 'exit')
        //   https://stackoverflow.com/a/12340730/7674702
        let poll_fd = PollFd::new(self.pty_result.master, PollFlags::POLLIN);

        if self.fd_drained {
            // Poll until the fd is ready to read again
            // This allows us to keep drawing frames and maintain a responsive
            // window even though reading the file is blocked.
            let poll_result = poll(&mut [poll_fd], FD_POLL_TIMEOUT_MS).unwrap();
            if poll_result == 0 {
                // We timed out, now is the time to draw a frame
                // We didn't actually read any data, so we'll send a zero
                *written = 0;
                return;
            }
            if poll_result == -1 {
                panic!("Poll failed");
            }
        }

        let read_count = self.get_master_file().read(buffer).unwrap();

        // TODO: Should we be pooling these up and sending them when the fd is drained?
        //   Currently, we'll be framerate limited when a program is printing more than 4K chars
        //   per 16ms, which is not a low threshold.
        *written = read_count;

        self.fd_drained = read_count != FD_BUFFER_SIZE_BYTES;
    }

    fn write(&mut self, data: &[u8]) {
        self.get_master_file().write_all(data).unwrap();
    }
}

impl MacOsShellLayer {
    pub fn new(rows: usize, cols: usize) -> Self {
        let winsize = winsize {
            // TODO: We should just report exactly how big we are, but we don't support
            //   stomping, so if we do that ZSH breaks.
            ws_col: cols as u16,
            ws_row: rows as u16,
            // TODO: Is this important?
            ws_xpixel: 100,
            ws_ypixel: 100,
        };
        // NOTE: This result does NOT implement Drop, so the file descriptors must be
        //   closed manually (or realistically, we will leak them).
        let pty_result = openpty(&winsize, None).expect("openpty() failed");

        let layer = MacOsShellLayer {
            pty_result,
            fd_drained: true,
            master_fd_file: None,
        };

        unsafe { layer.fork_and_become_shell_as_child_process() }
        // From now on we're in the parent process. The child uses exec to become the shell and
        // never reaches this point.

        layer
    }

    fn get_master_file(&mut self) -> &mut File {
        if self.master_fd_file.is_none() {
            self.master_fd_file = Some(unsafe { File::from_raw_fd(self.pty_result.master) });
        }
        self.master_fd_file.as_mut().unwrap()
    }

    unsafe fn fork_and_become_shell_as_child_process(&self) {
        let fork_result = fork();

        // Fork split our program in two, and now we check who we are
        // (at this point, both versions are running simulataneously, as though the
        //  disassembler in the USS Enterprise had broken down)
        match fork_result {
            // We are the parent in this instance, so we want nothing to do with this function
            Ok(ForkResult::Parent { .. }) => return,
            // We're the child, happy days
            Ok(ForkResult::Child) => {}
            // Oh crumbs.
            Err(_) => panic!("Process failed to fork"),
        }

        // The child process doesn't need a reference to the master PTY file.
        close(self.pty_result.master).unwrap();

        // Put the child process in a different process group to the parent.
        // This is required for the shell program to spawn sub-processes and keep track of the
        // foreground process for ^C, ^Z etc.
        setsid().unwrap();

        // Set the slave file to the "controlling terminal" of this process.
        // Slightly black-box magic to me.
        let pty_slave = self.pty_result.slave;
        ioctl::tiocsctty(pty_slave).unwrap();

        // Set basic io file descriptors for this process to read/write from the slave file
        dup2(pty_slave, 0).unwrap(); // StdIn
        dup2(pty_slave, 1).unwrap(); // StdOut
        dup2(pty_slave, 2).unwrap(); // StdErr

        // We no longer need this pointer to our slave fd (it's pointed to at 0, 1 and 2)
        close(pty_slave).unwrap();

        // See "man login". This program sets up some important env vars like $PATH and $HOME.
        // It also automatically spawns the user's preferred shell.
        let login_path = CString::new("/usr/bin/login").unwrap();
        // We use a special flag to tell login not to prompt us for a password, because we're
        // going to spawn it as the current user anyway.
        let login_force_flag = CString::new("-f").unwrap();
        // And then we pass the user's username as the argument for the force flag.
        let user_name = CString::new(whoami::username()).unwrap();

        // These are on top of the default ones, not in place of them
        let env_vars = [
            // This is very important, otherwise the shell won't talk to us properly
            // TODO: Eventually support xterm-256color
            CString::new("TERM=xterm-16color").unwrap(),
            // This is just showing off :)
            CString::new("TERM_PROGRAM=velocity").unwrap(),
        ];
        let mut c_env_vars: Vec<*const i8> = env_vars.iter().map(|s| s.as_ptr()).collect();
        // NULL-terminated
        c_env_vars.push(ptr::null());

        // The way the variadic arguments work here are:
        // 1 - The executable path
        // 2 through N - argv items
        // N+1 - NULL terminal for argv array
        // N+2 - Pointer to an array of environment variables
        // What it's doing is replacing our executable with the image at shell_path. So we the child
        // process *become* the shell program (maintaining the modifications we made to stdio above).
        // TODO: Panic if this is -1
        let _exec_result = execle(
            login_path.as_ptr(),
            login_path.clone().as_ptr(),
            login_force_flag.as_ptr(),
            user_name.as_ptr(),
            ptr::null() as *const c_void,
            c_env_vars.as_slice().as_ptr(),
        );
    }
}

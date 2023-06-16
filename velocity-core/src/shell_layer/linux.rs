// Written for Pop!_OS. May not work elsewhere
use super::ShellLayer;
use crate::constants::*;

use libc::{c_void, execle, winsize};
use nix::{
    poll::{poll, PollFd, PollFlags},
    pty::{openpty, OpenptyResult},
    unistd::{close, dup2, fork, setsid, ForkResult},
};
use std::{
    env,
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

pub struct LinuxShellLayer {
    pty_result: OpenptyResult,
    fd_drained: bool,
    master_fd_file: Option<File>,
}

impl ShellLayer for LinuxShellLayer {
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

impl LinuxShellLayer {
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

        let layer = LinuxShellLayer {
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

        // TODO: Find out which shell the user has set, rather than hardcoding zsh
        //   xterm does this by reading /etc/passwd, and executing /bin/sh if that doesn't work.
        let shell_path = CString::new("/usr/bin/zsh").unwrap();
        // Pass --login, which should set up extra env like $PATH
        // TODO: Offer the user the chance to choose whether they want a login shell or not.
        let shell_login_flag = CString::new("--login").unwrap();

        // Change to the user's home directory before spawning the shell program
        env::set_current_dir(env::var("HOME").unwrap_or("/".to_string())).unwrap();

        let mut env_vars = vec![];

        // First, we copy all of our own environment variables to this new process.
        // This will give it the important default ones like PATH etc.
        for var in env::vars() {
            // Skip these, we'll set our own
            if var.0 == "TERM" || var.0 == "TERMPROGRAM" {
                continue;
            }
            env_vars.push(CString::new(format!("{}={}", var.0, var.1)).unwrap())
        }

        // This is very important, otherwise the shell won't talk to us properly
        // NOTE: We only support 16 colours, but on the distro I tested with (Pop!_OS)
        //   16 colour support is not configured. It only recognises 256color as ANSI.
        env_vars.push(CString::new("TERM=xterm-256color").unwrap());
        // This is just showing off :)
        env_vars.push(CString::new("TERM_PROGRAM=velocity").unwrap());

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
            shell_path.as_ptr(),
            shell_path.clone().as_ptr(),
            shell_login_flag.as_ptr(),
            ptr::null() as *const c_void,
            c_env_vars.as_slice().as_ptr(),
        );
    }
}

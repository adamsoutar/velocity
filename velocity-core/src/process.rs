use crate::constants::*;
use libc::{c_void, execle, winsize};
use std::{ffi::CString, fs::File, io::Read, os::fd::FromRawFd, ptr};
// This file handles spawning the shell as a child process and hooking it up to
// a pseudoterminal.
use nix::{
    poll::{poll, PollFd, PollFlags},
    pty::openpty,
    unistd::{close, dup2, fork, setsid, ForkResult},
};

mod ioctl {
    ioctl_none_bad!(tiocsctty, libc::TIOCSCTTY);
}

pub unsafe fn fork_and_execute_shell() {
    let winsize = winsize {
        // Default size from iTerm2
        ws_col: 80,
        ws_row: 21,
        // TODO: Is this important?
        ws_xpixel: 100,
        ws_ypixel: 100,
    };
    // NOTE: This result does NOT implement Drop, so the file descriptors must be
    //   closed manually.
    let pty_result = openpty(&winsize, None).expect("openpty() failed");

    match fork() {
        Ok(ForkResult::Parent { .. }) => {
            // Read chunks from the child process forever
            let poll_fd = PollFd::new(pty_result.master, PollFlags::POLLIN);
            let mut buffer = [0u8; FD_BUFFER_SIZE_BYTES];
            let mut file = File::from_raw_fd(pty_result.master);
            let mut fd_drained = true;

            loop {
                if fd_drained {
                    // Poll until the fd is ready to read again
                    // This allows us to keep drawing frames and maintain a responsive
                    // window even though reading the file is blocked.
                    let poll_result = poll(&mut [poll_fd], FD_POLL_TIMEOUT_MS).unwrap();
                    if poll_result == 0 {
                        // We timed out, now is the time to draw a frame
                        println!("DRAW FRAME due to program suspension for 10ms");
                        continue;
                    }
                    if poll_result == -1 {
                        panic!("Poll failed");
                    }
                }

                println!("About to read the fd...");
                let read_count = file.read(&mut buffer).unwrap();
                println!("Read fd.");
                let read_string = std::str::from_utf8(&buffer[..read_count]).unwrap();

                println!("Parent: {}", read_string);

                if read_count != FD_BUFFER_SIZE_BYTES {
                    println!("DRAW FRAME due to I/O drain");
                    fd_drained = true;
                }
            }
        }
        Ok(ForkResult::Child) => become_shell_as_child_process(pty_result.master, pty_result.slave),
        Err(_) => panic!("fork() failed"),
    }
}

unsafe fn become_shell_as_child_process(pty_master: i32, pty_slave: i32) {
    // The child process doesn't need a reference to the master PTY file.
    close(pty_master).unwrap();

    // Put the child process in a different process group to the parent.
    // This is required for the shell program to spawn sub-processes and keep track of the
    // foreground process for ^C, ^Z etc.
    setsid().unwrap();

    // Set the slave file to the "controlling terminal" of this process.
    // Slightly black-box magic to me.
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

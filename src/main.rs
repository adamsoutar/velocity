#[macro_use]
extern crate nix;

mod constants;
mod process;

fn main() {
    unsafe {
        process::fork_and_execute_shell();
    }
}

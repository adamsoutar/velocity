use velocity_core::process::fork_and_execute_shell;

fn main() {
    unsafe { fork_and_execute_shell() }
}

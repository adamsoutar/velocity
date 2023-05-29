use velocity_core::shell_layer::get_shell_layer;

fn main() {
    let mut shell_layer = get_shell_layer();
    shell_layer.execute_and_run_shell(|data, len| {
        if len > 0 {
            let string = std::str::from_utf8(&data[..len]).unwrap();
            println!("{}", string);
        }
    });
}

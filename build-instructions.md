# Building velocity

First, you'll need a [Rust](https://rustlang.org) toolchain installed.

Next, you need to set up SFML for Rust by following [these instructions](https://github.com/jeremyletang/rust-sfml/wiki).

Finally, you can build and run velocity for yourself:

```bash
git clone https://github.com/adamsoutar/velocity
cd velocity/velocity-sfml
cargo run --release
```

If you're on a platform other than macOS, you will likely run into a handful of
small issues. You'll need to change the font path which is (for now) hardcoded
in `./src/main.rs`.

On certain distros, you might also need to start velocity as root due to an
issue with velocity's use of `/bin/login`.

```bash
cargo build --release &&
sudo ./target/release/velocity-sfml
```

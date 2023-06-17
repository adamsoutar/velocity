# Building velocity

First, you'll need a [Rust](https://rustlang.org) toolchain installed.

Next, you need to set up SFML for Rust by following [these instructions](https://github.com/jeremyletang/rust-sfml/wiki).

Finally, you can build and run velocity for yourself:

```bash
git clone https://github.com/adamsoutar/velocity
cd velocity/velocity-sfml
cargo run --release
```

This will just build and work on macOS and Pop!_OS. It probably works on all
Ubuntu-based things, and for that matter any Linux distro as long as you have Noto Mono installed.

I've tested it on macOS Ventura and Pop!_OS 22.04.
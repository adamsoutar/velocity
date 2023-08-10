# Building velocity

First, you'll need a [Rust](https://rustlang.org) toolchain installed. The next 
step depends on your preferred window manager.

## macOS or Linux with X11

Set up SFML for Rust by following 
[these instructions](https://github.com/jeremyletang/rust-sfml/wiki).

Then you can build and run Velocity for yourself:

```bash
git clone https://github.com/adamsoutar/velocity
cd velocity/velocity-sfml
cargo run --release
```

This will just work on macOS and Pop!_OS. It probably works on all Ubuntu-based 
things, and for that matter any Linux distro as long as you have Noto Mono 
installed.

I've tested it on macOS Ventura and Pop!_OS 22.04.

To generate a `.app` file with a nice icon and whatnot, use

```
cargo bundle --release
```

## Linux with Wayland

Velocity's main graphics backend is SFML. SFML only supports X11, so if you're
using Wayland without an X back-compat package, you can use the alternate SDL
backend.

Velocity on SDL is more of a proof of concept. It's missing support for a lot of
features, but you can try it out with:

```
git clone https://github.com/adamsoutar/velocity
cd velocity/velocity-sdl
cargo run --release
```

`velocity-sdl` does run on macOS for development purposes, but if you actually
intend to use it then `-sfml` is a much better choice.
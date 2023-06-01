## Rough plan

There should be a Trait which is implemented by each operating system's version of openpty(), fork(), evecle() etc.
Also for the file descriptor polling. Currently everything's very BSD/macOS specific.

The trait should cover everything the terminal emulator needs for spawning the shell, grabbing text bytes, and sending user input.

### GUI modules

Just like gbrs, we'll have modules for different GUI types.
SFML and SDL2 are the obvious ones, but it'd be nice to have very specfic native ones like Cocoa and Win32 too.

### Buffer structure

The scrollback buffer should be a VecDeque of lines, where each line is a VecDeque of characters. By characters we mean grapheme clusters, not bytes.

The scrollback_start var tells us how many lines to skip from the top of the scrollback buffer before we start drawing in the GUI.

### Escape code plan

We should have a TextState struct. This is what TtyState will use to determine
the styling of each character in the scrollback buffer. Bold, Italic, Colours,
etc. Then we should have an escape code parser which performs mutations on the
TextState.

In the escape code parser it'd be cool if it took in a text escape code and
returns one of those neat-o Rust enums that can handle arguments.

For example:

```rust
enum EscapeCode {
    Bold,
    Italic,
    Reset,
    ForegroundColour(Colour),
    BackgroundColour(Colour),
    BellTone(Frequency)
}
```

Couple of notes on escape code notes:

It *seeeeeems* like most if not all control codes end in a letter.
Unless they're something like `ESC 7` or `ESC 8`, but they can be detected
because they've got the space in them. NOTE: `ESC 7` and `ESC 8` are not
standardised. They're technically "private codes". But that doesn't mean we
shouldn't be prepared for them, because a program could chuck them out at
any point.

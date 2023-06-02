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

### Limiting scrollback

There should be a shared pool between all velocity windows of how much memory
they can spend on caching scrollback. At the same time, we should cache as much
as we can. Why not, after all?

So, I think about half of the system memory is fair. Eg. 8GB on a 16GB system.
Then if you have four velocity windows open, they can use 2GB each. After that
they just start purging lines because scrollback is a VecDeque

### ECMA Standard notation

01/11 is what the ECMA standard calls the ESC character.
That character is 0x1B. So their format must be decimal representations of each
byte with a slash in the middle. Why? Oh well.

So they claim that a CSI ends with any char in the range 04/00 to 07/14.

So that's 0x40 to 0x7E. In ASCII that's @ to ~, including all upper- and
lower-case letters. That would line up with what we've seen so far.

Parameter bytes are in the range 03/00 to 03/15. That's 0x30 to 0x3F. So that's
0 to ? in ASCII.

Intermediate bytes are in the range 02/00 to 02/15. That's 0x20 to 0x2F. So
that's Space to forward slash in ASCII.

### Crash

Currently, `git diff` in the velocity directory seems to crash the program.
I suspect my new line clearing code, because it's one of very few parts
of velocity that can actually panic.

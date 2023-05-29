## Rough plan

There should be a Trait which is implemented by each operating system's version of openpty(), fork(), evecle() etc.
Also for the file descriptor polling. Currently everything's very BSD/macOS specific.

The trait should cover everything the terminal emulator needs for spawning the shell, grabbing text bytes, and sending user input.

### GUI modules

Just like gbrs, we'll have modules for different GUI types.
SFML and SDL2 are the obvious ones, but it'd be nice to have very specfic native ones like Cocoa and Win32 too.

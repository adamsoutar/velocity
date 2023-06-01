#[derive(Debug)]
pub enum EscapeSequence {
    SetTextColour(TextColourArgs), // ESC[{x};{y};{z}m
    ResetAllTextStyles,            // ESC[0m
    EnableBoldText,                // ESC[1m
    DisableBoldText,               // ESC[22m
    EnableFaintText,               // ESC[2m
    DisableFaintText,              // ESC[22m (The same as bold?)
    EnableItalicText,              // ESC[3m
    DisableItalicText,             // ESC[23m
    EnableUnderlinedText,          // ESC[4m
    DisableUnderlinedText,         // ESC[24m
    EnableBlinkingText,            // ESC[5m
    DisableBlinkingText,           // ESC[25m
    EnableReverseVideoMode,        // ESC[7m
    DisableReverseVideoMode,       // ESC[27m
    EnableInvisibleText,           // ESC[8m
    DisableInvisibleText,          // ESC[28m
    EnableStrikethroughText,       // ESC[9m
    DisableStrikethroughText,      // ESC[29m
}

#[derive(Debug)]
pub struct TextColourArgs {
    // The text setting command comes with an additional sub-escape-sequence that
    // can do something else first, like enable bold text or reverse video.
    pub initial_sub_sequence: Box<EscapeSequence>,
    // It's followed by a variable-length sequence of colours which are, I think,
    // applied one by one.
    // How can there be more than a foreground and background, you say? Well,
    // there's a special reset colour which could be chucked in there before
    // setting fg/bg.
    pub colour_sequence: Vec<TerminalColour>,
}

#[derive(Debug)]
pub enum TerminalColour {
    // TODO: In future, we can define a special one like
    // RGBColour(RGBColourArgs) which can hold any colour
    // To start with, these are the 16-bit mode colours
    BlackForeground,   // 30
    BlackBackground,   // 40
    RedForeground,     // 31
    RedBackground,     // 41
    GreenForeground,   // 32
    GreenBackground,   // 42
    YellowForeground,  // 33
    YellowBackground,  // 43
    BlueForeground,    // 34
    BlueBackground,    // 44
    MagentaForeground, // 35
    MagentaBackground, // 45
    CyanForeground,    // 36
    CyanBackground,    // 46
    WhiteForeground,   // 37
    WhiteBackground,   // 47
    DefaultForeground, // 39
    DefaultBackground, // 49
    // NOTE: This "colour" acts like EscapeSequence::ResetAllTextStyles
    //   It turns off bold, etc. when used in a colour sequence.
    SpecialReset, // 0
}

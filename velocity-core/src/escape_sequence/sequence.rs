#[derive(Debug)]
pub enum EscapeSequence {
    // Sets the TextStyle with which we render things
    SelectGraphicRendition(Vec<SGRCode>), // ESC[...m
}

#[derive(FromPrimitive, Clone, Copy)]
pub enum TerminalColour {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    // TODO: Advanced(r, g, b)
    Default = 9,
    BrightBlack = 10,
    BrightRed = 11,
    BrightGreen = 12,
    BrightYellow = 13,
    BrightBlue = 14,
    BrightMagenta = 15,
    BrightCyan = 16,
    BrightWhite = 17,
}

// TODO: Move SGR stuff to its own file
#[derive(FromPrimitive, Debug, Clone, Copy, PartialEq)]
pub enum SGRCode {
    ResetAllTextStyles = 0,
    EnableBoldText = 1,
    EnableFaintText = 2,
    EnableItalicText = 3,
    EnableUnderlinedText = 4,
    EnableRapidBlinkingText = 5,
    EnableSlowBlinkingText = 6,
    EnableReverseVideoMode = 7,
    EnableInvisibleText = 8,
    EnableStrikethroughText = 9,
    SelectPrimaryFont = 10,
    SelectAlternativeFont1 = 11,
    SelectAlternativeFont2 = 12,
    SelectAlternativeFont3 = 13,
    SelectAlternativeFont4 = 14,
    SelectAlternativeFont5 = 15,
    SelectAlternativeFont6 = 16,
    SelectAlternativeFont7 = 17,
    SelectAlternativeFont8 = 18,
    SelectAlternativeFont9 = 19,
    EnableFrakturText = 20, // aka Gothic text
    EnableDoubleUnderlinedText = 21,
    ResetTextWeight = 22, // Disables bold and faint text
    ResetItalicText = 23,
    ResetUnderlinedText = 24,
    ResetBlinkingText = 25,
    ResetInverseVideoMode = 27,
    ResetInvisibleText = 28,
    ResetStrikethroughText = 29,
    SelectBlackForegroundColour = 30,
    SelectRedForegroundColour = 31,
    SelectGreenForegroundColour = 32,
    SelectYellowForegroundColour = 33,
    SelectBlueForegroundColour = 34,
    SelectMagentaForegroundColour = 35,
    SelectCyanForegroundColour = 36,
    SelectWhiteForegroundColour = 37,
    SelectAdvancedForegroundColour = 38, // For 256-colour mode and above
    SelectDefaultForegroundColour = 39,
    SelectBlackBackgroundColour = 40,
    SelectRedBackgroundColour = 41,
    SelectGreenBackgroundColour = 42,
    SelectYellowBackgroundColour = 43,
    SelectBlueBackgroundColour = 44,
    SelectMagentaBackgroundColour = 45,
    SelectCyanBackgroundColour = 46,
    SelectWhiteBackgroundColour = 47,
    SelectAdvancedBackgroundColour = 48, // For 256-colour mode and above
    SelectDefaultBackgroundColour = 49,
    ResetProportionalSpacing = 50, // From Wikipedia. Not really sure what this is.
    EnableFramedText = 51,
    EnableEncircledText = 52,
    EnableOverlinedText = 53,
    ResetFramedAndEncircledText = 54,
    ResetOverlinedText = 55,
    // 56 and 57 are not documented on Wikipedia
    SelectAdvancedUnderlineColour = 58,
    ResetAdvancedUnderlineColour = 59,
    // 60 to 75 are rare weird things
    SelectBrightBlackForegroundColour = 90,
    SelectBrightRedForegroundColour = 91,
    SelectBrightGreenForegroundColour = 92,
    SelectBrightYellowForegroundColour = 93,
    SelectBrightBlueForegroundColour = 94,
    SelectBrightMagentaForegroundColour = 95,
    SelectBrightCyanForegroundColour = 96,
    SelectBrightWhiteForegroundColour = 97,
    // 98 and 99 are undocumented
    SelectBrightBlackBackgroundColour = 100,
    SelectBrightRedBackgroundColour = 101,
    SelectBrightGreenBackgroundColour = 102,
    SelectBrightYellowBackgroundColour = 103,
    SelectBrightBlueBackgroundColour = 104,
    SelectBrightMagentaBackgroundColour = 105,
    SelectBrightCyanBackgroundColour = 106,
    SelectBrightWhiteBackgroundColour = 107,
}
use super::text_style::TextStyle;

#[derive(Clone, Copy)]
pub struct DecoratedChar {
    pub char: char,
    pub style: TextStyle,
}

impl DecoratedChar {
    pub fn new(char: char, style: TextStyle) -> Self {
        DecoratedChar { char, style }
    }
}

// Allows things like SFML's Text object to transparently read our decorated chars as chars
impl From<DecoratedChar> for char {
    fn from(decorated_char: DecoratedChar) -> Self {
        decorated_char.char
    }
}

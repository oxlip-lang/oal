use lsp_types::Position;

/// Returns the encoding index for the given UTF-16 text position.
pub(crate) fn index_for_position(len: fn(char) -> usize, text: &str, position: Position) -> usize {
    let mut line = 0;
    let mut character = 0;
    let mut index = 0;

    for c in text.chars() {
        if line == position.line {
            if character == position.character || c == '\n' || c == '\r' {
                break;
            }
            character += c.len_utf16() as u32;
        } else if c == '\n' {
            line += 1;
        }
        index += len(c);
    }

    index
}

/// Returns the UTF-16 text position for the given Unicode character index.
pub(crate) fn utf16_position(text: &str, index: usize) -> Position {
    let mut line = 0;
    let mut character = 0;

    for c in text.chars().take(index) {
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += c.len_utf16() as u32;
        }
    }

    Position { line, character }
}

/// Returns the Unicode _character_ index for the given UTF-16 text position.
pub(crate) fn char_index(text: &str, position: Position) -> usize {
    index_for_position(|_| 1, text, position)
}

/// Returns the UTF-8 index for the given UTF-16 text position.
pub(crate) fn utf8_index(text: &str, position: Position) -> usize {
    index_for_position(char::len_utf8, text, position)
}

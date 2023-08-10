use std::ops::Range;

/// Returns the UTF-8 index for the given UTF-16 text position.
pub(crate) fn position_to_utf8(text: &str, position: lsp_types::Position) -> usize {
    let mut line = 0;
    let mut character = 0;
    let mut utf8_index = 0;

    for c in text.chars() {
        if line == position.line {
            if character == position.character || c == '\n' || c == '\r' {
                break;
            }
            character += c.len_utf16() as u32;
        } else if c == '\n' {
            line += 1;
        }
        utf8_index += c.len_utf8();
    }

    utf8_index
}

/// Returns the UTF-16 text position for the given UTF-8 index.
pub(crate) fn utf8_to_position(text: &str, index: usize) -> lsp_types::Position {
    let mut line = 0;
    let mut character = 0;
    let mut utf8_index = 0;

    for c in text.chars() {
        if utf8_index >= index {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += c.len_utf16() as u32;
        }
        utf8_index += c.len_utf8();
    }

    lsp_types::Position { line, character }
}

/// Converts a UTF-8 range to a UTF-16 position range.
pub(crate) fn utf8_range_to_position(text: &str, range: Range<usize>) -> lsp_types::Range {
    let start = utf8_to_position(text, range.start);
    let end = utf8_to_position(text, range.end);
    lsp_types::Range { start, end }
}

#[test]
fn test_position_to_utf8() {
    assert_eq!('😉'.len_utf8(), 4);
    assert_eq!('😉'.len_utf16(), 2);
    let text = "hello\nworld\r\n😉text\r\n!";
    // The position of the character 'e' on the last line.
    let position = lsp_types::Position::new(2, 3);
    let index = position_to_utf8(&text, position);
    assert_eq!(index, 18);
}

#[test]
fn test_position_to_utf8_overflow() {
    let text = "hello\nworld\r\ntext\r\n!";
    // A position past the end of the last line.
    let position = lsp_types::Position::new(2, 10);
    let index = position_to_utf8(&text, position);
    // We expect the index of the carriage return at the end of the last line.
    assert_eq!(index, 17);
}

#[test]
fn test_utf8_to_position() {
    assert_eq!('😉'.len_utf16(), 2);
    let text = "hello\nworld\r\n😉text\r\n!";
    let index = 18; // The character 'e' in word "text"
    assert_eq!(&text[index..index + 1], "e");
    let position = utf8_to_position(&text, index);
    assert_eq!(position.line, 2);
    assert_eq!(position.character, 3);
}

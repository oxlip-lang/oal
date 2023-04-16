use crate::utf16::{char_index, utf16_position, utf8_index};
use lsp_types::Position;

#[test]
fn position_to_utf8() {
    assert_eq!('😉'.len_utf8(), 4);
    assert_eq!('😉'.len_utf16(), 2);
    let text = "hello\nworld\r\n😉text\r\n!";
    // The position of the character 'e' on the last line.
    let position = Position::new(2, 3);
    let index = utf8_index(&text, position);
    assert_eq!(index, 18);
}

#[test]
fn position_to_utf8_overflow() {
    let text = "hello\nworld\r\ntext\r\n!";
    // A position past the end of the last line.
    let position = Position::new(2, 10);
    let index = utf8_index(&text, position);
    // We expect the index of the carriage return at the end of the last line.
    assert_eq!(index, 17);
}

#[test]
fn position_to_char() {
    let text = "hello\nworld\r\ntext\r\n!";
    let position = Position::new(2, 10); // Position past the end of last line
    let index = char_index(&text, position);
    assert_eq!(index, 17);
}

#[test]
fn char_to_position() {
    assert_eq!('😉'.len_utf16(), 2);
    let text = "hello\nworld\r\n😉text\r\n!";
    let index = 15; // The character 'e' in word "text"
    assert_eq!(text.chars().skip(index).next().unwrap(), 'e');
    let position = utf16_position(&text, index);
    assert_eq!(position.line, 2);
    assert_eq!(position.character, 3);
}

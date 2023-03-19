use crate::utf16::{utf16_index, utf16_position, Text};
use lsp_types::Position;

#[test]
fn position_to_index() {
    let text = "hello\nworld\r\ntext\r\n!".encode_utf16().collect::<Text>();
    let position = Position::new(2, 10); // Position past the end of last line
    let index = utf16_index(&text, position).unwrap();
    assert_eq!(index, 17);
}

#[test]
fn index_to_position() {
    let text = "hello\nworld\r\ntext\r\n!";
    let index = 14; // The character 'e' in word "text"
    let position = utf16_position(&text, index).unwrap();
    assert_eq!(position.line, 2);
    assert_eq!(position.character, 1);
}

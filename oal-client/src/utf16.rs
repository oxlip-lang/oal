use anyhow::anyhow;
use lsp_types::Position;

/// Text encoded as UTF-16.
pub(crate) type Text = Vec<u16>;

/// Returns the UTF-16 index for the given text position.
pub(crate) fn utf16_index(text: &Text, position: Position) -> anyhow::Result<usize> {
    const NL: u16 = 10;
    const CR: u16 = 13;

    let offset = if position.line > 0 {
        text.iter()
            .enumerate()
            .filter_map(|(i, c)| if *c == NL { Some(i) } else { None })
            .enumerate()
            .skip_while(|(n, _i)| *n + 1 < position.line as usize)
            .map(|(_n, i)| i + 1)
            .next()
            .ok_or_else(|| anyhow!("position out of bounds"))?
    } else {
        0
    };

    let eol = text[offset..]
        .iter()
        .position(|&c| c == NL || c == CR)
        .unwrap_or(text.len());

    let column = std::cmp::min(eol, position.character as usize);

    Ok(offset + column)
}

/// Returns the UTF-16 text position for the given Unicode _character_ index.
pub(crate) fn utf16_position(text: &str, index: usize) -> anyhow::Result<Position> {
    let mut line = 0;
    let mut character = 0;

    for (_, c) in text.char_indices().take_while(|(i, _)| *i < index) {
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += c.len_utf16() as u32;
        }
    }

    Ok(Position { line, character })
}

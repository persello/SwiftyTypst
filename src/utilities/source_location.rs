use typst::syntax::Source;

pub struct SourceLocation {
    pub byte_offset: u64,
    pub line: u64,
    pub column: u64,
}

impl SourceLocation {
    pub(crate) fn from_byte_offset(offset: usize, source: &Source) -> Option<Self> {
        let line = source.byte_to_line(offset)?;
        let column = source.byte_to_column(offset)?;

        Some(Self {
            byte_offset: offset as u64,
            line: line as u64,
            column: column as u64,
        })
    }
}

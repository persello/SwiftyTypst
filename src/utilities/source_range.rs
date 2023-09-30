use std::ops::Range;

use typst::syntax::Source;

use super::source_location::SourceLocation;

pub struct SourceRange {
    pub start: SourceLocation,
    pub end: SourceLocation,
}

impl SourceRange {
    pub(crate) fn from_range(range: Range<usize>, source: &Source) -> Option<Self> {
        let start = SourceLocation::from_byte_offset(range.start, source)?;
        let end = SourceLocation::from_byte_offset(range.end, source)?;

        Some(Self { start, end })
    }
}

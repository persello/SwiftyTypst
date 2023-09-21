use typst::{eval::Bytes, font::Font};

pub struct FontDefinition {
    pub data: Vec<u8>,
}

impl IntoIterator for FontDefinition {
    type Item = Font;

    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        let buffer = Bytes::from(self.data);
        Font::iter(buffer).collect::<Vec<_>>().into_iter()
    }
}

pub trait FontReader: Send + Sync {
    fn fonts(&self) -> Vec<FontDefinition>;
}

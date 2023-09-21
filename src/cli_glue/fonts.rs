use crate::TypstCompiler;

use typst::{eval::Bytes, font::Font};

impl TypstCompiler {
    pub fn add_font(&self, font: FontDefinition) {
        let fonts = font.into_iter();

        for font in fonts {
            if let Ok(mut world) = self.world.write() {
                world.book.update(|book| book.push(font.info().clone()));
                let index = world.fonts.len();
                world.fonts.insert(index, font);
            }
        }
    }
}

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

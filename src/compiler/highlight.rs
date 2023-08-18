use std::{ops::Range, path::PathBuf};

use typst::{
    ide::Tag,
    syntax::{FileId, LinkedNode},
    util::PathExt,
    World,
};

use super::TypstCompiler;

pub struct HighlightResult {
    pub start: u64,
    pub end: u64,
    pub tag: Tag,
}

impl TypstCompiler {
    pub fn highlight(&self, file_path: String) -> Vec<HighlightResult> {
        let path = PathBuf::from(file_path);
        let Some(real_path) = self.world.read().unwrap().root.join_rooted(&path) else {
            return vec![];
        };

        self.world.write().unwrap().reset();

        let id = FileId::new(None, &real_path);
        let source = self.world.read().unwrap().source(id).unwrap();

        let node = LinkedNode::new(source.root());

        self.highlight_tree(&node)
            .iter()
            .map(|r| HighlightResult {
                start: r.0.start as u64,
                end: r.0.end as u64,
                tag: r.1,
            })
            .collect()
    }

    fn highlight_tree(&self, node: &LinkedNode) -> Vec<(Range<usize>, Tag)> {
        let mut tags = vec![];

        if let Some(tag) = typst::ide::highlight(node) {
            tags.push((node.range(), tag));
        }

        for child in node.children() {
            tags.append(&mut self.highlight_tree(&child));
        }

        tags
    }
}

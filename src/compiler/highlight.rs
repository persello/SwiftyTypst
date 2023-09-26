use std::{ops::Range, path::PathBuf};

use typst::{
    ide::Tag,
    syntax::{FileId, LinkedNode, VirtualPath},
    World,
};

use super::TypstCompiler;

pub struct HighlightResult {
    pub start: u64,
    pub end: u64,
    pub tag: Tag,
}

impl TypstCompiler {
    pub fn highlight(&self, file_path: String) {
        let compiler = self.clone();
        std::thread::spawn(move || {
            let path = PathBuf::from(file_path.clone());
            let vpath = VirtualPath::new(path);

            compiler.world.write().unwrap().reset();

            let id = FileId::new(None, vpath);
            let Ok(source) = compiler.world.read().unwrap().source(id) else {
                return;
            };

            let node = LinkedNode::new(source.root());

            let result: Vec<HighlightResult> = compiler
                .highlight_tree(&node)
                .iter()
                .map(|r| HighlightResult {
                    start: source.byte_to_utf16(r.0.start).unwrap() as u64,
                    end: source.byte_to_utf16(r.0.end).unwrap() as u64,
                    tag: r.1,
                })
                .collect();

            compiler
                .delegate
                .lock()
                .unwrap()
                .highlighting_finished(file_path, result);
        });
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

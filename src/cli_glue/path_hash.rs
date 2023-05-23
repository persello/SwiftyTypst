use std::{path::Path, hash::Hash};

use siphasher::sip128::{SipHasher13, Hasher128};
use typst::diag::{FileResult};

/// A hash that is the same for all paths pointing to the same entity.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PathHash(u128);

impl PathHash {
    pub fn new(path: &Path) -> FileResult<Self> {
        let mut state = SipHasher13::new();
        path.hash(&mut state);
        Ok(Self(state.finish128().as_u128()))
    }
}
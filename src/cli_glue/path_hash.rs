use std::{hash::Hash, path::Path};

use siphasher::sip128::{Hasher128, SipHasher13};
use typst::{diag::FileResult, syntax::PackageSpec};

/// A hash that is the same for all paths pointing to the same entity.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct PathHash(u128);

impl PathHash {
    pub fn new(path: &Path, package: &Option<&PackageSpec>) -> FileResult<Self> {
        let mut state = SipHasher13::new();
        path.hash(&mut state);
        package.hash(&mut state);
        Ok(Self(state.finish128().as_u128()))
    }
}

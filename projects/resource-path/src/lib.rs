use std::{
    path::{Path, PathBuf},
    str::FromStr,
};

use url::{ParseError, Url};

mod third_party;

/// A path to a resource, either local or remote.
#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ResourcePath {
    /// The network path to the resource.
    pub remote: Url,
    /// The local path to the resource.
    pub local: PathBuf,
}

impl ResourcePath {
    /// Create a new resource path to link remote and local object.
    pub fn new<N, L>(remote: N, local: L) -> Result<Self, ParseError>
    where
        N: AsRef<str>,
        L: AsRef<Path>,
    {
        Ok(Self { remote: Url::from_str(remote.as_ref())?, local: local.as_ref().to_path_buf() })
    }
    /// Creates a new resource path.
    pub fn with_local<P: AsRef<Path>>(mut self, local: P) -> Self {
        self.local = local.as_ref().to_path_buf();
        self
    }
    /// Creates a new resource path.
    pub fn with_remote<P: AsRef<str>>(mut self, remote: P) -> Result<Self, ParseError> {
        self.remote = Url::from_str(remote.as_ref())?;
        Ok(self)
    }
}

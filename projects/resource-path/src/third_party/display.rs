use std::{
    fmt::{Debug, Display, Formatter},
    path::PathBuf,
    str::FromStr,
};

use url::{ParseError, Url};

use crate::ResourcePath;

impl Debug for ResourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourcePath").field("network", &self.remote.as_str()).field("local", &self.local).finish()
    }
}

impl Display for ResourcePath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}?local={}", self.remote, self.local.display())
    }
}

impl FromStr for ResourcePath {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut url = Url::parse(s)?;
        let mut local = PathBuf::from(".");
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "local" | "relative" => local = PathBuf::from(value.as_ref()),
                _ => {}
            }
        }
        url.set_query(None);
        Ok(Self { remote: url, local })
    }
}

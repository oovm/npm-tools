use crate::ResourcePath;
use trauma::download::Download;

impl ResourcePath {
    /// Convert path to a download task.
    pub fn as_download_task(&self) -> Download {
        Download { url: self.remote.clone(), filename: self.local.to_string_lossy().to_string() }
    }
}

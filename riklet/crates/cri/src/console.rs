use crate::*;
use log::warn;
use std::path::{Path, PathBuf};
use tokio::net::UnixListener;

/// An implementation of a PTY socket
pub struct ConsoleSocket {
    socket_path: PathBuf,
    listener: Option<UnixListener>,
}

impl ConsoleSocket {
    pub fn new(socket_path: &Path) -> Result<Self> {
        let listener = UnixListener::bind(socket_path).context(UnixSocketOpenError {})?;
        debug!("UnixListener binded on {}", &socket_path.to_str().unwrap());
        Ok(Self {
            socket_path: socket_path.to_path_buf(),
            listener: Some(listener),
        })
    }

    pub fn get_listener(&self) -> &Option<UnixListener> {
        &self.listener
    }
}

/// Implement Drop trait.
/// The drop() method will be called when the struct is going out of the scope in order to delete the socket file.
impl Drop for ConsoleSocket {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.socket_path) {
            warn!("Failed to clean up console socket : {}", e)
        }
    }
}

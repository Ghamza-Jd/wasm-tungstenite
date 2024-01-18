pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("URL scheme not supported")]
    UnsupportedUrlScheme,
    #[error("Connection closed")]
    ConnectionClosed,
}

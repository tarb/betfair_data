use super::stream_types::StreamErrorCode;

#[derive(Debug)]
pub enum Error {
    IO(std::io::Error),
    Parse(serde_json::Error),
    Stream { code: StreamErrorCode, msg: String },
    Unexpected,
    EmptyMessage,
    NotLoggedIn,
}

impl From<Error> for pyo3::PyErr {
    fn from(_: Error) -> Self {
        unimplemented!()
    }
}
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::Parse(err)
    }
}
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::IO(err)
    }
}
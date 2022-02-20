use std::path::PathBuf;

pub struct DeserErr {
    pub file: PathBuf,
    pub err: serde_json::Error,
}

pub struct IOErr {
    pub file: Option<PathBuf>,
    pub err: std::io::Error,
}
use std::path::PathBuf;

pub struct IOErr {
    pub file: Option<PathBuf>,
    pub err: std::io::Error,
}

pub struct DataError {
    pub missing_field: &'static str,
}

use std::io;
pub struct HttpError {
    pub message: String,
    pub err_data: Vec<u8>,
}

impl From<io::Error> for HttpError {
    fn from(error: io::Error) -> Self {
        HttpError {
            message: error.to_string(),
            err_data: vec![],
        }
    }
}
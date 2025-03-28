use std::io;

#[derive(Debug)]
pub enum Error {
    IO(io::Error),
    InvalidCommand,
    InvalidArguments,
    InvalidDirection,
    EndOfFile,
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

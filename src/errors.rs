use std::convert;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct MissingSongError {}

impl Error for MissingSongError {}

impl fmt::Display for MissingSongError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

impl convert::From<std::io::Error> for MissingSongError {
    fn from(_error: std::io::Error) -> MissingSongError {
        MissingSongError {}
    }
}

impl convert::From<hound::Error> for MissingSongError {
    fn from(_error: hound::Error) -> MissingSongError {
        MissingSongError {}
    }
}

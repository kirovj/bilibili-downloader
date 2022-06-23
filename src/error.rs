use std::{fmt, io};

#[derive(Debug)]
pub struct Brror {
    kind: BrrorKind,
    message: String,
}

#[derive(Debug)]
enum BrrorKind {
    Net,
    IO,
}

impl fmt::Display for Brror {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} error, {}", self.kind, self.message)
    }
}

impl From<ureq::Error> for Brror {
    fn from(error: ureq::Error) -> Self {
        Brror {
            kind: BrrorKind::Net,
            message: error.to_string(),
        }
    }
}

impl From<io::Error> for Brror {
    fn from(error: io::Error) -> Self {
        Brror {
            kind: BrrorKind::IO,
            message: error.to_string(),
        }
    }
}

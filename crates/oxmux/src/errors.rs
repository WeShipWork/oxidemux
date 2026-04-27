use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    NotImplemented { boundary: &'static str },
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotImplemented { boundary } => {
                write!(formatter, "{boundary} core behavior is not implemented yet")
            }
        }
    }
}

impl std::error::Error for CoreError {}

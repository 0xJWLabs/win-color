/// Represents errors that can occur when handling colors in Windows.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ErrorKind {
    /// Error when data is invalid.
    InvalidData,
    // Error when input is invalid.
    InvalidInput,
    // Error when unknown.
    InvalidUnknown,
}

impl core::fmt::Display for ErrorKind {
    /// Formats the `ErrorKind` enum into a human-readable string for display purposes.
    ///
    /// This is used when the error kind is displayed directly (e.g., in an error message).
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidUnknown => write!(f, "invalid unknown format"),
            Self::InvalidInput => write!(f, "invalid input"),
            Self::InvalidData => write!(f, "invalid data"),
        }
    }
}

#[derive(Clone)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl core::fmt::Debug for Error {
    fn fmt(&self, fmt: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut debug = fmt.debug_struct("Error");
        debug
            .field("kind", &self.kind())
            .field("message", &self.message())
            .finish()
    }
}

impl Eq for Error {}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl core::hash::Hash for Error {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl PartialOrd for Error {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Error {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.kind.cmp(&other.kind)
    }
}

impl Error {
    /// Creates a new `Error` instance with a specified kind and a message.
    /// If the message is empty, it will use an empty message for the error.
    ///
    /// # Parameters
    /// - `kind`: The type of error (e.g., `InvalidGradientCoordinates`, `InvalidAccent`).
    /// - `message`: A message describing the error. This can be an empty string.
    ///
    /// # Returns
    /// A new `Error` instance with the specified error kind and message.
    pub fn new<T: AsRef<str>>(kind: ErrorKind, message: T) -> Self {
        let message: &str = message.as_ref();
        Self {
            kind,
            message: message.to_string(),
        }
    }

    /// Retrieves the kind of the error.
    ///
    /// # Returns
    /// A reference to the `ErrorKind` variant that represents the type of error.
    pub fn kind(&self) -> ErrorKind {
        self.kind.clone()
    }

    /// Retrieves the error message, if provided.
    ///
    /// # Returns
    /// The error message as a string slice. If no message is provided, an empty string is returned.
    pub fn message(&self) -> String {
        self.message.clone()
    }
}

impl core::fmt::Display for Error {
    /// Formats the `Error` struct for user-facing display.
    ///
    /// If a message is provided, it includes the message along with the error kind.
    /// If no message is provided, only the error kind is displayed.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        if self.message.is_empty() {
            write!(f, "{}", self.kind)
        } else {
            write!(f, "{} ({})", self.kind, self.message)
        }
    }
}

impl std::error::Error for Error {}

/// A custom `Result` type that returns `Error` in case of failure.
///
/// This type is used for handling errors related to shell operations. It wraps the standard `Result` type but replaces the error type with our custom `Error` type.
pub type Result<T> = core::result::Result<T, Error>;

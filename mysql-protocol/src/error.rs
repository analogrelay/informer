use std::borrow::Cow;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
  ConnectionReset,
  DataIncomplete,
  InvalidPacket,
  NotSupported,
  ProtocolError,
  ServerIncapable,
  ClientIncapable,
  UnsupportedProtocol { required: u8, requested: u8 },
  Other
}

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    description: Cow<'static, str>,
    cause: Option<Box<dyn std::error::Error>>
}

impl Error {
    pub fn new<S: Into<Cow<'static, str>>>(kind: ErrorKind, description: S) -> Error {
        Error { kind, description: description.into(), cause: None }
    }

    pub fn with_cause<S: Into<Cow<'static, str>>, E: Into<Box<dyn std::error::Error>>>(kind: ErrorKind, description: S, cause: E) -> Error {
        Error {
            kind,
            description: description.into(),
            cause: Some(cause.into()),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.cause.as_ref().map(|e| e.as_ref())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?} ({})", self.kind, self.description.as_ref())
    }
}

impl std::cmp::PartialEq for Error {
  fn eq(&self, other: &Error) -> bool {
      match (self.kind, other.kind) {
          (ErrorKind::Other, _) => false,
          (_, ErrorKind::Other) => false,
          (l, r) => l.eq(&r),
      }
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Error {
    Error::with_cause(ErrorKind::Other, format!("{}", e), e)
  }
}

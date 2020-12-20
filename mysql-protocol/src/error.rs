use std::borrow::Cow;

#[derive(Debug)]
pub enum Error {
  ConnectionReset,
  DataIncomplete,
  InvalidPacket(Cow<'static, str>),
  NotSupported(Cow<'static, str>),
  Other(Box<dyn std::error::Error>)
}

impl std::error::Error for Error {
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ConnectionReset => write!(f, "connection reset by peer"),
            Error::DataIncomplete => write!(f, "insufficient data received"),
            Error::InvalidPacket(s) => write!(f, "invalid packet: {}", s),
            Error::NotSupported(s) => write!(f, "not supported: {}", s),
            Error::Other(e) => write!(f, "{}", e)
        }
    }
}

impl std::cmp::PartialEq for Error {
  fn eq(&self, other: &Error) -> bool {
    match (self, other) {
      (Error::ConnectionReset, Error::ConnectionReset) => true,
      (Error::DataIncomplete, Error::DataIncomplete) => true,
      (Error::InvalidPacket(s1), Error::InvalidPacket(s2)) => s1 == s2,
      (Error::NotSupported(s1), Error::NotSupported(s2)) => s1 == s2,

      // Notable: Error::Other never equals Error::Other
      (_, _) => false,
    }
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Error {
    Error::Other(e.into())
  }
}

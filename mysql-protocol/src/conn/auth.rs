use std::todo;
use crate::error::Error;

#[derive(Debug)]
pub enum AuthPlugin {
    CachingSha2Password,
    Unknown(String)
}

impl AuthPlugin {
    pub fn from_name(name: impl AsRef<str> + Into<String>) -> AuthPlugin {
        match name.as_ref() {
            "caching_sha2_password" => AuthPlugin::CachingSha2Password,
            _ => AuthPlugin::Unknown(name.into())
        }
    }

    pub fn name(&self) -> &str {
        match self {
            AuthPlugin::CachingSha2Password => "caching_sha2_password",
            AuthPlugin::Unknown(x) => x.as_str()
        }
    }

    pub fn generate_response(&self, password: Option<&str>, _server_data: &[u8]) -> Result<Vec<u8>, Error> {
        if let Some(_) = password {
            todo!("password hashing")
        } else {
            Ok(Vec::new())
        }
    }
}

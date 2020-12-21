use mysql_common::constants::CapabilityFlags;

#[derive(Debug, Default)]
pub struct ConnectionOptions {
    username: Option<String>,
    password: Option<String>,
    initial_database: Option<String>,
    use_ssl: bool
}

impl ConnectionOptions {
    pub fn build() -> ConnectionOptionsBuilder {
        ConnectionOptionsBuilder::new()
    }

    pub fn username(&self) -> Option<&str> { self.username.as_ref().map(|s| s.as_str()) }
    pub fn password(&self) -> Option<&str> { self.password.as_ref().map(|s| s.as_str()) }
    pub fn initial_database(&self) -> Option<&str> { self.initial_database.as_ref().map(|s| s.as_str()) }
    pub fn use_ssl(&self) -> bool { self.use_ssl }

    pub fn get_capabilities(&self) -> CapabilityFlags {
        let mut caps = CapabilityFlags::from_bits_truncate(0);
        if self.initial_database.is_some() {
            caps |= CapabilityFlags::CLIENT_CONNECT_WITH_DB
        }

        if self.use_ssl {
            caps |= CapabilityFlags::CLIENT_SSL
        }

        caps
    }
}

pub struct ConnectionOptionsBuilder {
    username: Option<String>,
    password: Option<String>,
    initial_database: Option<String>,
    use_ssl: bool
}

impl ConnectionOptionsBuilder {
    fn new() -> ConnectionOptionsBuilder {
        ConnectionOptionsBuilder {
            username: None,
            password: None,
            initial_database: None,
            use_ssl: false
        }
    }

    pub fn build(self) -> ConnectionOptions {
        ConnectionOptions {
            username: self.username,
            password: self.password,
            initial_database: self.initial_database,
            use_ssl: self.use_ssl,
        }
    }

    pub fn username<S: Into<String>>(mut self, user: S) -> ConnectionOptionsBuilder {
        self.username = Some(user.into());
        self
    }

    pub fn password<S: Into<String>>(mut self, pass: S) -> ConnectionOptionsBuilder {
        self.password = Some(pass.into());
        self
    }

    pub fn initial_database<S: Into<String>>(mut self, db: S) -> ConnectionOptionsBuilder {
        self.initial_database = Some(db.into());
        self
    }

    pub fn use_ssl(mut self, ssl: bool) -> ConnectionOptionsBuilder {
        self.use_ssl = ssl;
        self
    }
}

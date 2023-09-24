use log::info;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct SerializedConfig {
    address: String,
    data_path: String,
    persistance: String,
}

pub struct Config {
    pub address: std::net::SocketAddr,
    pub data_path: String,
    pub persistance: String,
}

impl Config {
    pub fn read(path: &str) -> Result<Self, ConfigError> {
        info!("Reading config from: {}", path);
        let file =
            std::fs::File::open(path).map_err(|err| ConfigError::file(String::from(path), err))?;
        let serde_config: SerializedConfig = serde_json::from_reader(file)?;
        Self::try_from(serde_config)
    }
}

impl TryFrom<SerializedConfig> for Config {
    type Error = ConfigError;

    fn try_from(
        SerializedConfig {
            address,
            data_path,
            persistance,
        }: SerializedConfig,
    ) -> Result<Self, Self::Error> {
        let address = address.parse()?;
        Ok(Config {
            address,
            data_path,
            persistance,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    File { path: String, inner: std::io::Error },
    Parsing { inner: serde_json::Error },
    Address { inner: std::net::AddrParseError },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::File { path, inner } => write!(f, "could not read ({}): {}", path, inner),
            ConfigError::Parsing { inner } => write!(f, "could not parse: {}", inner),
            ConfigError::Address { inner } => write!(f, "bad address: {}", inner),
        }
    }
}

impl std::error::Error for ConfigError {}

impl ConfigError {
    fn file(path: String, inner: std::io::Error) -> Self {
        ConfigError::File { path, inner }
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(inner: serde_json::Error) -> Self {
        ConfigError::Parsing { inner }
    }
}

impl From<std::net::AddrParseError> for ConfigError {
    fn from(inner: std::net::AddrParseError) -> Self {
        ConfigError::Address { inner }
    }
}

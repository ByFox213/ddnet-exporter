use env_logger::Builder;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    #[serde(default = "default_logging", alias = "RUST_LOG", alias = "logging")]
    pub logging: String,

    #[serde(
        default = "default_port",
        alias = "PORT",
        alias = "port",
        alias = "web_port"
    )]
    pub web_port: u16,

    #[serde(default = "default_delay", alias = "DELAY", alias = "delay")]
    pub delay: u64,

    #[serde(default = "default_timeout", alias = "TIMEOUT", alias = "timeout")]
    pub timeout: u64,

    #[serde(
        default = "default_error_delay",
        alias = "ERROR_DELAY",
        alias = "error_delay"
    )]
    pub error_delay: u64,
}

fn default_logging() -> String {
    "INFO".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_delay() -> u64 {
    15
}

fn default_timeout() -> u64 {
    15
}

fn default_error_delay() -> u64 {
    5
}

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
    }

    pub fn set_logging(&self) {
        let mut builder = Builder::new();
        builder.parse_filters(&self.logging);
        let _ = builder.try_init();
    }
}

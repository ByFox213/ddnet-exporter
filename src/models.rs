use env_logger::Builder;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "RUST_LOG")]
    #[serde(default = "default_logging")]
    pub logging: String,

    #[serde(rename = "PORT")]
    #[serde(default = "default_port")]
    pub web_port: u16,

    #[serde(rename = "DELAY")]
    #[serde(default = "default_delay")]
    pub delay: u64,
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

impl Config {
    pub fn from_env() -> Result<Self, envy::Error> {
        envy::from_env::<Self>()
    }

    pub fn set_logging(&self) {
        let mut builder = Builder::new();
        builder.parse_filters(&self.logging);
        builder.init();
    }
}

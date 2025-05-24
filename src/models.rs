use env_logger::Builder;
use log::LevelFilter;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "RUST_LOG")]
    pub logging: Option<String>,
    
    #[serde(rename = "PORT")]
    #[serde(default = "default_port")]
    pub web_port: u16,
    
    #[serde(rename = "DELAY")]
    #[serde(default = "default_delay")]
    pub delay: u64,
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
        println!("{:?}", self);
        let mut builder = Builder::new();
        builder.filter_level(LevelFilter::Info);
        if self.logging.is_some() {
            builder.parse_filters(&self.logging.clone().unwrap());
        }
        builder.init();
    }
}
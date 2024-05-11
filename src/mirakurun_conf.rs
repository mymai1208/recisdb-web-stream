use std::fs;

use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Channel {
    pub name: String,
    #[serde(rename = "type")]
    pub r#type: String,
    pub channel: String,
    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tuner {
    pub name: String,
    pub types: Vec<String>,
    pub command: String,
    #[serde(rename = "isDisabled")]
    pub is_disabled: bool,
}

pub struct MirakurunConfing {
    pub channels: Vec<Channel>,
    pub tuners: Vec<Tuner>,
}

impl MirakurunConfing {
    pub fn new() -> Self {
        Self {
            channels: Vec::new(),
            tuners: Vec::new(),
        }
    }

    pub fn load_mirakurun_conf(&mut self) -> Result<(), std::io::Error> {
        info!("Loading mirakurun config");

        let channels_raw = fs::read_to_string("./channels.yml")?;
        let tuners_raw = fs::read_to_string("./tuners.yml")?;

        self.channels
            .append(&mut serde_yaml::from_str::<Vec<Channel>>(&channels_raw).unwrap());
        self.tuners
            .append(&mut serde_yaml::from_str::<Vec<Tuner>>(&tuners_raw).unwrap());

        info!("Loaded {} channels", self.channels.len());
        info!("Loaded {} tuners", self.tuners.len());

        Ok(())
    }
}

impl Default for MirakurunConfing {
    fn default() -> Self {
        Self::new()
    }
}

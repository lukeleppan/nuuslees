use std::{collections::HashMap, fmt, path::PathBuf};

use color_eyre::eyre::Result;
use config::Value;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use derive_deref::{Deref, DerefMut};
use ratatui::style::{Color, Modifier, Style};
use serde::{
  de::{self, Deserializer, MapAccess, Visitor},
  Deserialize, Serialize,
};
use serde_json::Value as JsonValue;

use crate::{action::Action, mode::Mode};

#[derive(Clone, Debug, Deserialize, Default)]
pub struct AppConfig {
  #[serde(default)]
  pub _data_dir: PathBuf,
  #[serde(default)]
  pub _config_dir: PathBuf,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Config {
  #[serde(default, flatten)]
  pub config: AppConfig,
  #[serde(default = "default_as_true")]
  pub confirm_quit: bool,
  #[serde(default)]
  pub groups: Vec<GroupConfig>,
}

impl Config {
  pub fn new() -> Result<Self, config::ConfigError> {
    let data_dir = crate::utils::get_data_dir();
    let config_dir = crate::utils::get_config_dir();
    let mut builder = config::Config::builder()
      .set_default("_data_dir", data_dir.to_str().unwrap())?
      .set_default("_config_dir", config_dir.to_str().unwrap())?;

    let config_files = [("config.toml", config::FileFormat::Toml)];
    let mut found_config = false;
    for (file, format) in &config_files {
      builder = builder.add_source(config::File::from(config_dir.join(file)).format(*format).required(false));
      if config_dir.join(file).exists() {
        found_config = true
      }
    }
    if !found_config {
      log::error!("No configuration file found. Application may not behave as expected");
    }

    let cfg: Self = builder.build()?.try_deserialize()?;

    Ok(cfg)
  }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct GroupConfig {
  pub name: String,
  pub desc: String,
  pub feeds: Vec<FeedConfig>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FeedConfig {
  pub name: Option<String>,
  pub desc: Option<String>,
  pub link: String,
}

const fn default_as_true() -> bool {
  true
}
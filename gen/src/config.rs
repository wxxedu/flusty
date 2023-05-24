use std::{error::Error, fmt::Display, path::PathBuf};

use serde::{Deserialize, Serialize};

const DEFAULT_RUST_ENTRY: &str = "native/src/";
const DEFAULT_DART_OUT: &str = "lib/gen/";
const DEFAULT_CONFIG_DIR: &str = "flusty.toml";

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Config {
    rust_entry: Option<String>,
    dart_out: Option<String>,
}

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Toml(toml::de::Error),
    PathNotFound(PathBuf),
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        Self::Toml(e)
    }
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "IO error: {}", e),
            ConfigError::Toml(e) => write!(f, "TOML error: {}", e),
            ConfigError::PathNotFound(path) => {
                write!(f, "Path not found: {}", path.display())
            }
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Toml(e) => Some(e),
            ConfigError::PathNotFound(_) => None,
        }
    }
}

unsafe impl Send for ConfigError {}

unsafe impl Sync for ConfigError {}

impl Config {
    fn root_dir() -> PathBuf {
        let cwd = std::env::current_dir()
            .expect("Failed to get current working directory")
            .to_str()
            .expect("Failed to convert current working directory to string")
            .to_string();
        let mut path = PathBuf::from(&cwd);
        loop {
            if path.join(DEFAULT_CONFIG_DIR).exists() {
                return path;
            }
            if !path.pop() {
                break;
            }
        }
        cwd.into()
    }

    pub fn from_disk() -> Result<Self, ConfigError> {
        let mut path = Self::root_dir();
        if !path.join(DEFAULT_CONFIG_DIR).exists() {
            return Ok(Self::default());
        }
        path.push(DEFAULT_CONFIG_DIR);
        let toml = std::fs::read_to_string(path)?;
        Self::from_toml(&toml)
    }

    fn from_toml(toml: &str) -> Result<Self, ConfigError> {
        toml::from_str(toml).map_err(|e| e.into())
    }

    pub fn rust_entry(&self) -> PathBuf {
        let root = Self::root_dir();
        root.join(
            self.rust_entry
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(DEFAULT_RUST_ENTRY),
        )
    }

    pub fn dart_out(&self) -> PathBuf {
        let root = Self::root_dir();
        root.join(
            self.dart_out
                .as_ref()
                .map(|s| s.as_str())
                .unwrap_or(DEFAULT_DART_OUT),
        )
    }

    pub fn set_rust_entry(&mut self, path: &str) {
        self.rust_entry = Some(path.to_string());
    }

    pub fn set_dart_out(&mut self, path: &str) {
        self.dart_out = Some(path.to_string());
    }

    pub fn load(&mut self) -> Result<&mut Self, ConfigError> {
        let root = Self::root_dir();
        let path = root.join(DEFAULT_CONFIG_DIR);
        if !path.exists() {
            return Err(ConfigError::PathNotFound(path));
        }
        let toml = std::fs::read_to_string(path)?;
        let config = Self::from_toml(&toml)?;
        self.rust_entry = config.rust_entry;
        self.dart_out = config.dart_out;
        Ok(self)
    }

    pub fn save(&self) {
        let root = Self::root_dir();
        let toml = toml::to_string(self).expect("Failed to serialize config");
        std::fs::write(root.join(DEFAULT_CONFIG_DIR), toml)
            .expect("Failed to write config to disk");
    }
}

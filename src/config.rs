use directories::ProjectDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct Config {
    pub calendar_path: Option<String>,
}

impl Config {
    fn config_dir() -> Option<PathBuf> {
        ProjectDirs::from("com", "calchemy", "calchemy").map(|d| d.config_dir().to_path_buf())
    }

    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|d| d.join("config.toml"))
    }

    pub fn load() -> Option<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            return None;
        }

        let content = fs::read_to_string(&config_path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn default_calendar_path() -> PathBuf {
        if let Some(proj_dirs) = ProjectDirs::from("com", "calchemy", "calchemy") {
            let data_dir = proj_dirs.data_dir();
            fs::create_dir_all(data_dir).ok();
            data_dir.join("events.cal")
        } else {
            PathBuf::from("events.cal")
        }
    }

    pub fn calendar_path(&self) -> PathBuf {
        if let Some(ref path) = self.calendar_path {
            if path.starts_with('~')
                && let Ok(home) = std::env::var("HOME")
            {
                return PathBuf::from(path.replacen('~', &home, 1));
            }
            PathBuf::from(path)
        } else {
            Self::default_calendar_path()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calendar_path_with_absolute_path() {
        let config = Config {
            calendar_path: Some("/absolute/path/events.cal".to_string()),
        };
        assert_eq!(
            config.calendar_path(),
            PathBuf::from("/absolute/path/events.cal")
        );
    }

    #[test]
    fn test_calendar_path_with_tilde_shorthand() {
        let config = Config {
            calendar_path: Some("~/cal/events.cal".to_string()),
        };
        let path = config.calendar_path();
        assert!(path.starts_with(std::path::Path::new("/")));
        assert!(path.ends_with("cal/events.cal"));
    }

    #[test]
    fn test_calendar_path_none_falls_back_to_default() {
        let config = Config { calendar_path: None };
        let path = config.calendar_path();
        assert!(path.to_string_lossy().contains("events.cal"));
    }

    #[test]
    fn test_load_parses_valid_toml() {
        let toml_content = r#"calendar_path = "/custom/path/events.cal""#;
        let parsed: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(parsed.calendar_path, Some("/custom/path/events.cal".to_string()));
    }

    #[test]
    fn test_load_parses_empty_config() {
        let toml_content = "";
        let parsed: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(parsed.calendar_path, None);
    }
}

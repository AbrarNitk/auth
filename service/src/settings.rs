#[derive(Debug, serde::Deserialize)]
pub struct Settings {
    pub service: ServiceSettings,
    #[serde(rename = "redis")]
    pub redis: base::ctx::settings::redis::RedisSettings,
    pub telemetry: base::ctx::settings::telemetry::TelemetrySettings,
    // #[serde(rename = "openfga")]
    // pub(crate) openfga: OpenFGASettings,
    #[serde(rename = "dex")]
    pub dex: base::ctx::settings::dex::DexSettings,

    pub postgres: Pg,
}

#[derive(Debug, serde::Deserialize)]
pub struct Pg {
    pub url: String,
}

#[derive(Debug, serde::Deserialize)]
pub struct ServiceSettings {
    pub environment: String,
    pub bind: String,
    pub port: u16,
}

impl Settings {
    pub fn new_with_file(path: &str, env: &str) -> Result<Self, config::ConfigError> {
        let settings_file_name = if path.is_empty() {
            format!("settings-{}.toml", env)
        } else {
            format!("{}/settings-{}.toml", path, env)
        };

        let default_settings = if path.is_empty() {
            "default.toml".to_string()
        } else {
            format!("{}/default.toml", path)
        };

        let settings = config::Config::builder()
            .add_source(config::File::with_name(default_settings.as_str()))
            .add_source(config::File::with_name(settings_file_name.as_str()))
            .build()?;
        settings.try_deserialize()
    }
}

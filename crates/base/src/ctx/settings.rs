pub mod dex {
    #[derive(Debug, serde::Deserialize, Clone)]
    pub struct DexSettings {
        #[serde(rename = "client-id")]
        pub client_id: String,
        #[serde(rename = "client-secret")]
        pub client_secret: String,
        #[serde(rename = "issuer-url")]
        pub issuer_url: String,
        #[serde(rename = "token-url")]
        pub token_url: String,
        #[serde(rename = "callback-url")]
        pub callback_url: String,
        pub connectors: Vec<DexConnector>,
        pub scopes: Vec<String>,
    }

    #[derive(Debug, serde::Deserialize, Clone)]
    pub struct DexConnector {
        pub id: String,
        #[serde(rename = "type")]
        pub r#type: String,
        pub name: String,
    }
}

pub mod openfga {
    #[derive(Clone, Debug, serde::Deserialize)]
    pub struct OpenFGASettings {
        pub host: String,
        pub api_key: Option<String>,
        pub store_id: String,
        pub auth_model_id: String,
    }
}

pub mod redis {
    #[derive(Debug, serde::Deserialize)]
    pub struct RedisSettings {
        pub url: String,
        #[serde(rename = "connection-timeout")]
        pub connection_timeout: Option<u64>,
        #[serde(rename = "max-size")]
        pub max_size: Option<u32>,
        #[serde(rename = "min-idle")]
        pub min_idle: Option<u32>,
        #[serde(rename = "idle-timeout")]
        pub idle_timeout: Option<u64>,
    }
}

pub mod telemetry {

    #[derive(Debug, serde::Deserialize)]
    pub struct TelemetrySettings {
        #[serde(rename = "otel-exporter-otlp-protocol")]
        pub otel_exporter_otlp_protocol: String,
        #[serde(rename = "otel-exporter-otlp-endpoint")]
        pub otel_exporter_otlp_endpoint: String,
        #[serde(rename = "otel-service-name")]
        pub otel_service_name: String,
        #[serde(rename = "log-level")]
        pub log_level: String,
        #[serde(rename = "username")]
        pub username: String,
        #[serde(rename = "password")]
        pub password: String,
        #[serde(rename = "org-name")]
        pub org_name: String,
        #[serde(rename = "stream-name")]
        pub stream_name: String,
    }
}

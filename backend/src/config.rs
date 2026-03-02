use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub access_token_expiry_secs: u64,
    pub refresh_token_expiry_secs: u64,
}

fn require_env(key: &str) -> Result<String, ConfigError> {
    std::env::var(key).map_err(|_| ConfigError::MissingEnvVar(key.to_string()))
}

fn env_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn parse_env<T: std::str::FromStr>(key: &str, value: &str) -> Result<T, ConfigError> {
    value.parse::<T>().map_err(|_| ConfigError::InvalidValue {
        key: key.to_string(),
        message: format!("cannot parse '{value}'"),
    })
}

impl AppConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load .env file if it exists (ignore errors)
        let _ = dotenvy::dotenv();

        let database_url = require_env("DATABASE_URL")?;
        let jwt_secret = require_env("JWT_SECRET")?;

        let host = env_or("SERVER_HOST", "0.0.0.0");
        let port_str = env_or("SERVER_PORT", "3000");
        let port = parse_env::<u16>("SERVER_PORT", &port_str)?;

        let max_conn_str = env_or("DATABASE_MAX_CONNECTIONS", "5");
        let max_connections = parse_env::<u32>("DATABASE_MAX_CONNECTIONS", &max_conn_str)?;

        let redis_url = env_or("REDIS_URL", "redis://127.0.0.1:6379");

        let access_str = env_or("JWT_ACCESS_TOKEN_EXPIRY_SECS", "3600");
        let access_token_expiry_secs = parse_env::<u64>("JWT_ACCESS_TOKEN_EXPIRY_SECS", &access_str)?;

        let refresh_str = env_or("JWT_REFRESH_TOKEN_EXPIRY_SECS", "604800");
        let refresh_token_expiry_secs = parse_env::<u64>("JWT_REFRESH_TOKEN_EXPIRY_SECS", &refresh_str)?;

        Ok(Self {
            server: ServerConfig { host, port },
            database: DatabaseConfig {
                url: database_url,
                max_connections,
            },
            redis: RedisConfig { url: redis_url },
            jwt: JwtConfig {
                secret: jwt_secret,
                access_token_expiry_secs,
                refresh_token_expiry_secs,
            },
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(String),

    #[error("Invalid value for {key}: {message}")]
    InvalidValue { key: String, message: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests that mutate env vars must run serially to avoid flaky races.
    static ENV_MUTEX: std::sync::LazyLock<std::sync::Mutex<()>> =
        std::sync::LazyLock::new(|| std::sync::Mutex::new(()));

    #[test]
    fn loads_config_from_env_vars() {
        let _guard = ENV_MUTEX.lock().unwrap();
        // Set up environment variables
        unsafe {
            std::env::set_var("SERVER_HOST", "127.0.0.1");
            std::env::set_var("SERVER_PORT", "8080");
            std::env::set_var("DATABASE_URL", "postgres://user:pass@localhost:5432/feedback");
            std::env::set_var("DATABASE_MAX_CONNECTIONS", "10");
            std::env::set_var("REDIS_URL", "redis://localhost:6379");
            std::env::set_var("JWT_SECRET", "test-secret-key-for-testing");
            std::env::set_var("JWT_ACCESS_TOKEN_EXPIRY_SECS", "3600");
            std::env::set_var("JWT_REFRESH_TOKEN_EXPIRY_SECS", "604800");
        }

        let config = AppConfig::from_env().expect("Should load config from env");

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
        assert_eq!(config.database.url, "postgres://user:pass@localhost:5432/feedback");
        assert_eq!(config.database.max_connections, 10);
        assert_eq!(config.redis.url, "redis://localhost:6379");
        assert_eq!(config.jwt.secret, "test-secret-key-for-testing");
        assert_eq!(config.jwt.access_token_expiry_secs, 3600);
        assert_eq!(config.jwt.refresh_token_expiry_secs, 604800);
    }

    #[test]
    fn uses_default_values_when_optional_vars_missing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        // Only set required variables
        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://user:pass@localhost:5432/feedback");
            std::env::set_var("JWT_SECRET", "test-secret-key");
            // Remove optional ones
            std::env::remove_var("SERVER_HOST");
            std::env::remove_var("SERVER_PORT");
            std::env::remove_var("DATABASE_MAX_CONNECTIONS");
            std::env::remove_var("REDIS_URL");
            std::env::remove_var("JWT_ACCESS_TOKEN_EXPIRY_SECS");
            std::env::remove_var("JWT_REFRESH_TOKEN_EXPIRY_SECS");
        }

        let config = AppConfig::from_env().expect("Should load config with defaults");

        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.database.max_connections, 5);
        assert_eq!(config.redis.url, "redis://127.0.0.1:6379");
        assert_eq!(config.jwt.access_token_expiry_secs, 3600);
        assert_eq!(config.jwt.refresh_token_expiry_secs, 604800);
    }

    #[test]
    fn returns_error_when_required_vars_missing() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::remove_var("DATABASE_URL");
            std::env::remove_var("JWT_SECRET");
        }

        let result = AppConfig::from_env();
        assert!(result.is_err());
    }

    #[test]
    fn returns_error_for_invalid_port() {
        let _guard = ENV_MUTEX.lock().unwrap();
        unsafe {
            std::env::set_var("DATABASE_URL", "postgres://localhost/test");
            std::env::set_var("JWT_SECRET", "secret");
            std::env::set_var("SERVER_PORT", "not_a_number");
        }

        let result = AppConfig::from_env();
        assert!(result.is_err());
    }
}

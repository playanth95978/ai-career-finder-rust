use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub app_name: String,
    pub app_env: String,
    pub app_port: u16,
    pub app_host: String,
    /// Whether to use HTTPS for generated URLs (e.g., OAuth2 redirects)
    pub app_https: bool,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    /// Directory to serve static files from (for serving SPA UI)
    pub static_files_dir: Option<String>,
    /// Whether to enable static file serving
    pub serve_static_files: bool,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            app_name: env::var("APP_NAME").unwrap_or_else(|_| "jobSearchRust".to_string()),
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            app_port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .expect("APP_PORT must be a valid port number"),
            app_host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            app_https: env::var("APP_HTTPS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
            database_url: env::var("DATABASE_URL")
                .expect("DATABASE_URL must be set"),
            jwt_secret: env::var("JWT_SECRET")
                .expect("JWT_SECRET must be set"),
            jwt_expiration_hours: env::var("JWT_EXPIRATION_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXPIRATION_HOURS must be a valid number"),
            static_files_dir: env::var("STATIC_FILES_DIR").ok(),
            serve_static_files: env::var("SERVE_STATIC_FILES")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or_else(|_| env::var("STATIC_FILES_DIR").is_ok()),
        }
    }

    pub fn is_production(&self) -> bool {
        self.app_env == "production"
    }

    pub fn is_development(&self) -> bool {
        self.app_env == "development"
    }
}

// Track 1 Phase 1b (2026-05-11): from_env() coverage. Mutex serializes
// env-var tests; rest of the suite stays parallel.
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn clear_app_env() {
        for k in [
            "APP_NAME",
            "APP_ENV",
            "APP_PORT",
            "APP_HOST",
            "APP_HTTPS",
            "STATIC_FILES_DIR",
            "SERVE_STATIC_FILES",
            "DATABASE_URL",
            "JWT_SECRET",
            "JWT_EXPIRATION_HOURS",
        ] {
            std::env::remove_var(k);
        }
    }

    /// Set the env vars `from_env()` calls `.expect()` on, so tests that
    /// exercise OTHER vars don't panic. Lets each test focus on what it
    /// actually asserts.
    fn set_required_env() {
        std::env::set_var("DATABASE_URL", "postgres://test:test@localhost:5432/test");
        std::env::set_var("JWT_SECRET", "test-secret-not-used");
    }

    #[test]
    fn test_from_env_uses_documented_defaults() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        let cfg = AppConfig::from_env();
        clear_app_env();
        // app_name default is baseName — assert non-empty for cross-scaffold portability.
        assert!(!cfg.app_name.is_empty());
        assert_eq!(cfg.app_env, "development");
        assert_eq!(cfg.app_port, 8080);
        assert_eq!(cfg.app_host, "0.0.0.0");
    }

    #[test]
    fn test_from_env_reads_app_port_from_env() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("APP_PORT", "9090");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.app_port, 9090);
    }

    #[test]
    fn test_from_env_reads_app_name_from_env() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("APP_NAME", "my-overridden-app");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.app_name, "my-overridden-app");
    }

    #[test]
    fn test_from_env_reads_app_host_from_env() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("APP_HOST", "127.0.0.1");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.app_host, "127.0.0.1");
    }

    #[test]
    fn test_from_env_reads_app_env_from_env() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("APP_ENV", "production");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.app_env, "production");
        assert!(cfg.is_production());
    }

    #[test]
    fn test_from_env_reads_database_url() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("DATABASE_URL", "postgres://user:pw@host/db");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.database_url, "postgres://user:pw@host/db");
    }


    #[test]
    fn test_from_env_reads_jwt_secret() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("JWT_SECRET", "real-secret-value");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.jwt_secret, "real-secret-value");
    }

    #[test]
    fn test_from_env_jwt_expiration_hours_default_24() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.jwt_expiration_hours, 24);
    }

    #[test]
    fn test_from_env_jwt_expiration_hours_override() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("JWT_EXPIRATION_HOURS", "48");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.jwt_expiration_hours, 48);
    }


    #[test]
    fn test_from_env_app_https_default_false() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert!(!cfg.app_https);
    }

    #[test]
    fn test_from_env_app_https_true() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("APP_HTTPS", "true");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert!(cfg.app_https);
    }

    #[test]
    fn test_from_env_static_files_dir_none_when_unset() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert!(cfg.static_files_dir.is_none());
        // serve_static_files defaults to false when STATIC_FILES_DIR is unset.
        assert!(!cfg.serve_static_files);
    }

    #[test]
    fn test_from_env_static_files_dir_implies_serve_static_files_true() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("STATIC_FILES_DIR", "/var/www");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert_eq!(cfg.static_files_dir.as_deref(), Some("/var/www"));
        // SERVE_STATIC_FILES unset, STATIC_FILES_DIR set => defaults to true.
        assert!(cfg.serve_static_files);
    }

    #[test]
    fn test_from_env_serve_static_files_explicit_overrides_dir_implicit() {
        let _g = ENV_LOCK.lock().unwrap();
        clear_app_env();
        set_required_env();
        std::env::set_var("STATIC_FILES_DIR", "/var/www");
        std::env::set_var("SERVE_STATIC_FILES", "false");
        let cfg = AppConfig::from_env();
        clear_app_env();
        assert!(!cfg.serve_static_files);
    }

    #[test]
    fn test_is_production_true() {
        let config = AppConfig {
            app_name: "test".to_string(),
            app_env: "production".to_string(),
            app_port: 8080,
            app_host: "0.0.0.0".to_string(),
            app_https: false,
            database_url: "test.db".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_expiration_hours: 24,
            static_files_dir: None,
            serve_static_files: false,
        };
        assert!(config.is_production());
        assert!(!config.is_development());
    }

    #[test]
    fn test_is_development_true() {
        let config = AppConfig {
            app_name: "test".to_string(),
            app_env: "development".to_string(),
            app_port: 8080,
            app_host: "0.0.0.0".to_string(),
            app_https: false,
            database_url: "test.db".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_expiration_hours: 24,
            static_files_dir: None,
            serve_static_files: false,
        };
        assert!(config.is_development());
        assert!(!config.is_production());
    }

    #[test]
    fn test_is_neither_production_nor_development() {
        let config = AppConfig {
            app_name: "test".to_string(),
            app_env: "staging".to_string(),
            app_port: 8080,
            app_host: "0.0.0.0".to_string(),
            app_https: false,
            database_url: "test.db".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_expiration_hours: 24,
            static_files_dir: None,
            serve_static_files: false,
        };
        assert!(!config.is_production());
        assert!(!config.is_development());
    }

    #[test]
    fn test_app_config_clone() {
        let config = AppConfig {
            app_name: "test".to_string(),
            app_env: "development".to_string(),
            app_port: 8080,
            app_host: "localhost".to_string(),
            app_https: false,
            database_url: "test.db".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_expiration_hours: 24,
            static_files_dir: None,
            serve_static_files: false,
        };
        let cloned = config.clone();
        assert_eq!(cloned.app_name, config.app_name);
        assert_eq!(cloned.app_port, config.app_port);
    }

    #[test]
    fn test_app_config_debug() {
        let config = AppConfig {
            app_name: "test_app".to_string(),
            app_env: "development".to_string(),
            app_port: 3000,
            app_host: "127.0.0.1".to_string(),
            app_https: false,
            database_url: "test.db".to_string(),
            jwt_secret: "secret".to_string(),
            jwt_expiration_hours: 24,
            static_files_dir: None,
            serve_static_files: false,
        };
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("test_app"));
        assert!(debug_str.contains("3000"));
    }
}

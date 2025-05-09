use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_name: String,
    pub environment: Environment,
    pub database_url: String,
    pub redis_url: Option<String>,
    pub uploads_dir: String,
    pub max_file_size: usize,
    pub api_base_url: String,
    pub media_base_url: String,
    pub jwt_secret: String,
    pub jwt_expiry: i64,
}

/// Environment where the application is running in
#[derive(Debug, Clone, PartialEq)]
pub enum Environment {
    Development,
    Testing,
    Staging,
    Production,
}

impl Environment {
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "production" => Environment::Production,
            "staging" => Environment::Staging,
            "testing" => Environment::Testing,
            _ => Environment::Development,
        }
    }
    
    pub fn is_dev(&self) -> bool {
        matches!(self, Environment::Development)
    }
    
    pub fn is_prod(&self) -> bool {
        matches!(self, Environment::Production)
    }
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let app_name = env::var("APP_NAME")
            .unwrap_or_else(|_| "eventsphere-be".to_string());
            
        let environment = Environment::from_str(
            &env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string())
        );
            
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set");
            
        let redis_url = env::var("REDIS_URL").ok();
            
        let uploads_dir = env::var("UPLOADS_DIR")
            .unwrap_or_else(|_| "uploads".to_string());
            
        let max_file_size = env::var("MAX_FILE_SIZE")
            .unwrap_or_else(|_| "2097152".to_string()) // 2MB default
            .parse::<usize>()
            .expect("MAX_FILE_SIZE must be a valid number");
            
        let api_base_url = env::var("API_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000/api/v1".to_string());
            
        let media_base_url = env::var("MEDIA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:8000/uploads".to_string());
            
        let jwt_secret = env::var("JWT_SECRET")
            .expect("JWT_SECRET must be set");
            
        let jwt_expiry = env::var("JWT_EXPIRY")
            .unwrap_or_else(|_| "86400".to_string()) // 24 hours default
            .parse::<i64>()
            .expect("JWT_EXPIRY must be a valid number");
            
        Self {
            app_name,
            environment,
            database_url,
            redis_url,
            uploads_dir,
            max_file_size,
            api_base_url,
            media_base_url,
            jwt_secret,
            jwt_expiry,
        }
    }
}
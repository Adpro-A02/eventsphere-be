use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Initialize the application logger
pub fn init_logger() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    // File logging (if LOG_DIR is specified)
    if let Ok(log_dir) = env::var("LOG_DIR") {
        let file_appender = RollingFileAppender::new(
            Rotation::DAILY,
            log_dir,
            "application.log",
        );
        
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().with_writer(std::io::stdout))
            .with(tracing_subscriber::fmt::layer().with_writer(non_blocking))
            .init();
        
        Box::leak(Box::new(_guard));
    } else {
        // Console-only logging
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }
}
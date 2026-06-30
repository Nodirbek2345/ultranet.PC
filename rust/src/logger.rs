use tracing_subscriber::{fmt, EnvFilter};
use tracing_appender::rolling;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_logger() {
    INIT.call_once(|| {
        let file_appender = rolling::daily("logs", "ultranet.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
            .with_writer(non_blocking)
            .with_ansi(false)
            .init();
        
        tracing::info!("UltraNet AI Logger initialized.");
    });
}

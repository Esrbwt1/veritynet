// Re-export log macros for convenience
pub use log::{debug, error, info, trace, warn};

// Public function to initialize logging (can be called by node binary later)
pub fn setup_logging(log_level: Option<&str>) {
    let log_level = log_level.unwrap_or("info"); // Default to info level
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or(log_level))
        .init();
    info!("VerityNet Core logging setup complete. Log Level: {}", log_level);
}

// Example function to demonstrate usage
pub fn core_hello() {
    info!("Hello from VerityNet Core!");
    debug!("This is a debug message from core.");
}

// Basic tests module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // Setup logging for tests (optional, but can be useful)
        // Use `try_init` to avoid panic if logging is already initialized
        let _ = env_logger::builder().is_test(true).try_init();

        info!("Running test: it_works");
        let result = 2 + 2;
        assert_eq!(result, 4);
        core_hello(); // Call our example function
        debug!("Test completed successfully.");
    }
}
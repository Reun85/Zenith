pub fn get_logger_level() -> tracing::Level {
    #[cfg(build_type = "dist")]
    {
        tracing::Level::WARN
    }

    #[cfg(build_type = "release")]
    {
        tracing::Level::DEBUG
    }

    #[cfg(build_type = "debug")]
    {
        tracing::Level::TRACE
    }
}


#[must_use]
pub const fn get_logger_level() -> tracing::Level {
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
    #[cfg(not(any(build_type = "debug", build_type = "release", build_type = "dist")))]
    {
        tracing::Level::ERROR
    }
}

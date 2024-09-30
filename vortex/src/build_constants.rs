pub fn get_logger_level() -> tracing::Level {
    #[cfg(build_type = "dist")]
    {
        return tracing::Level::WARN;
    }

    #[cfg(build_type = "release")]
    {
        return tracing::Level::DEBUG;
    }

    #[cfg(build_type = "debug")]
    {
        return tracing::Level::TRACE;
    }
    #[cfg(not(any(build_type = "debug", build_type = "release", build_type = "dist")))]
    {
        return tracing::Level::ERROR;
    }
}

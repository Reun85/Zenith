pub use tracing::{
    debug, debug_span, error, error_span, event, info, info_span, span, trace, trace_span, warn,
    warn_span, Level,
};

pub extern crate tracing;

#[derive(Copy, Clone, PartialEq, Eq, Debug, smart_default::SmartDefault)]
#[allow(clippy::struct_excessive_bools)]
pub struct LoggingCreateInfo {
    #[default(Level::INFO)]
    pub level: Level,
    #[default(false)]
    pub span_enter: bool,
    #[default(false)]
    pub span_leave: bool,
    #[default(false)]
    pub with_file: bool,
    #[default(false)]
    pub with_line_number: bool,
}
impl LoggingCreateInfo {
    #[must_use]
    pub const fn max() -> Self {
        Self {
            level: Level::TRACE,
            span_enter: true,
            span_leave: true,
            with_file: true,
            with_line_number: true,
        }
    }
    #[must_use]
    pub const fn min() -> Self {
        Self {
            level: Level::ERROR,
            span_enter: false,
            span_leave: false,
            with_file: false,
            with_line_number: false,
        }
    }
}

#[derive(Debug, derive_more::From, thiserror::Error)]
pub enum Error {
    #[error("Failed to set global default")]
    SetGlobalDefault(tracing::subscriber::SetGlobalDefaultError),
}

#[must_use]
pub const fn get_logger_level_based_on_build_type() -> tracing::Level {
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
/// # Errors
/// Returns whether the initializion failed
pub fn init_logging() -> Result<(), Error> {
    create(LoggingCreateInfo {
        level: get_logger_level_based_on_build_type(),
        ..LoggingCreateInfo::max()
    })?;
    info!("Logging initialized");
    Ok(())
}

/// # Errors
/// Returns whether the initializion failed
pub fn create(info: LoggingCreateInfo) -> Result<(), Error> {
    use tracing_subscriber::fmt::format::FmtSpan;
    let LoggingCreateInfo {
        level,
        span_enter,
        span_leave,
        with_file,
        with_line_number: with_file_number,
    } = info;
    let int = if span_enter {
        FmtSpan::ENTER
    } else {
        FmtSpan::NONE
    } | if span_leave {
        FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };
    tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            .with_file(with_file)
            .with_line_number(with_file_number)
            .with_span_events(int)
            .with_max_level(level)
            .finish(),
    )
    .map_err(Into::into)
}

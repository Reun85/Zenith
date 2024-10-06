#![allow(dead_code, unused_imports)]
pub use tracing::{
    debug, debug_span, error, error_span, event, info, info_span, span, trace, trace_span, warn,
    warn_span, Level,
};

pub extern crate tracing;

#[derive(Copy, Clone, PartialEq, Eq, Debug, smart_default::SmartDefault)]
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
    pub fn max() -> Self {
        Self {
            level: Level::TRACE,
            span_enter: true,
            span_leave: true,
            with_file: true,
            with_line_number: true,
        }
    }
    pub fn min() -> Self {
        Self {
            level: Level::ERROR,
            span_enter: false,
            span_leave: false,
            with_file: false,
            with_line_number: false,
        }
    }
}

pub fn create(info: LoggingCreateInfo) {
    let LoggingCreateInfo {
        level,
        span_enter,
        span_leave,
        with_file,
        with_line_number: with_file_number,
    } = info;
    use tracing_subscriber::fmt::format::FmtSpan;
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
    .expect("setting default subscriber failed");
}

//! Armature Logging Framework
//!
//! Provides structured logging for the Armature framework with JSON output
//! by default, and configurable pretty-printing for development.
//!
//! # Features
//!
//! - **JSON by default**: Production-ready structured logging
//! - **Pretty printing**: Human-readable output for development
//! - **Environment-controlled**: Configure via environment variables
//! - **Zero-cost when disabled**: Debug macros compile to no-ops in release
//! - **Runtime configurable**: Change format/level at runtime
//!
//! # Quick Start
//!
//! ```rust
//! use armature_log::{debug, info, warn, error};
//!
//! // Default: JSON output
//! info!("Server started on port {}", 8080);
//! // {"timestamp":"2024-12-20T12:00:00Z","level":"INFO","target":"my_app","message":"Server started on port 8080"}
//!
//! // With target
//! debug!(target: "armature::router", "Matching route: {}", "/api/users");
//! ```
//!
//! # Switching to Pretty Logging
//!
//! ## Option 1: Environment Variable (Recommended)
//!
//! ```bash
//! # Pretty format for development
//! ARMATURE_LOG_FORMAT=pretty cargo run
//!
//! # JSON format for production (default)
//! cargo run
//!
//! # Compact format
//! ARMATURE_LOG_FORMAT=compact cargo run
//! ```
//!
//! ## Option 2: Programmatic Configuration
//!
//! ```rust,no_run
//! use armature_log::{configure, Format, Level};
//!
//! // Configure for development
//! configure()
//!     .format(Format::Pretty)
//!     .level(Level::Debug)
//!     .color(true)
//!     .apply();
//!
//! // Or use presets
//! armature_log::preset_development();  // Pretty + Debug + Colors
//! armature_log::preset_production();   // JSON + Info
//! ```
//!
//! # Environment Variables
//!
//! | Variable | Values | Default | Description |
//! |----------|--------|---------|-------------|
//! | `ARMATURE_DEBUG` | `1`, `true` | `false` | Enable debug logging |
//! | `ARMATURE_LOG_LEVEL` | `trace`, `debug`, `info`, `warn`, `error` | `info` | Minimum log level |
//! | `ARMATURE_LOG_FORMAT` | `json`, `pretty`, `compact` | `json` | Output format |
//! | `ARMATURE_LOG_COLOR` | `1`, `true`, `0`, `false` | auto-detect | Enable ANSI colors |
//! | `ARMATURE_LOG_TIMESTAMPS` | `1`, `0` | `1` | Include timestamps |
//! | `ARMATURE_LOG_MODULE` | `1`, `0` | `1` | Include module path |
//!
//! # Output Formats
//!
//! ## JSON (Default)
//! ```text
//! {"timestamp":"2024-12-20T12:00:00Z","level":"INFO","target":"my_app","message":"Server started"}
//! ```
//!
//! ## Pretty
//! ```text
//! 2024-12-20 12:00:00.123 INFO  my_app Server started
//! ```
//!
//! ## Compact
//! ```text
//! 12:00:00 I my_app: Server started
//! ```

use once_cell::sync::Lazy;
use std::env;
use std::io::Write;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

// ============================================================================
// Log Levels
// ============================================================================

/// Log level for Armature logging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum Level {
    /// Trace level (most verbose)
    Trace = 0,
    /// Debug level
    Debug = 1,
    /// Info level
    Info = 2,
    /// Warning level
    Warn = 3,
    /// Error level (least verbose)
    Error = 4,
    /// Off (no logging)
    Off = 5,
}

impl Level {
    /// Parse level from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "trace" => Some(Level::Trace),
            "debug" => Some(Level::Debug),
            "info" => Some(Level::Info),
            "warn" | "warning" => Some(Level::Warn),
            "error" => Some(Level::Error),
            "off" | "none" => Some(Level::Off),
            _ => None,
        }
    }

    /// Get level name.
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Trace => "TRACE",
            Level::Debug => "DEBUG",
            Level::Info => "INFO",
            Level::Warn => "WARN",
            Level::Error => "ERROR",
            Level::Off => "OFF",
        }
    }

    /// Get colored level name (if color feature enabled).
    #[cfg(feature = "color")]
    pub fn colored(&self) -> colored::ColoredString {
        use colored::Colorize;
        match self {
            Level::Trace => "TRACE".magenta(),
            Level::Debug => "DEBUG".blue(),
            Level::Info => "INFO".green(),
            Level::Warn => "WARN".yellow(),
            Level::Error => "ERROR".red().bold(),
            Level::Off => "OFF".white(),
        }
    }
}

impl std::str::FromStr for Level {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

impl std::fmt::Display for Level {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Log Format
// ============================================================================

/// Output format for log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Format {
    /// Pretty format with colors (default for TTY)
    Pretty = 0,
    /// Compact single-line format
    Compact = 1,
    /// JSON format for structured logging
    Json = 2,
}

impl Format {
    /// Parse format from string.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pretty" => Some(Format::Pretty),
            "compact" => Some(Format::Compact),
            "json" => Some(Format::Json),
            _ => None,
        }
    }
}

impl std::str::FromStr for Format {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s).ok_or(())
    }
}

// ============================================================================
// Global Configuration
// ============================================================================

/// Global debug flag - checked by macros.
static DEBUG_ENABLED: AtomicBool = AtomicBool::new(false);

/// Global log level.
static LOG_LEVEL: AtomicU8 = AtomicU8::new(Level::Info as u8);

/// Global configuration (lazy initialized).
static CONFIG: Lazy<LogConfig> = Lazy::new(LogConfig::from_env);

/// Logging configuration.
#[derive(Debug)]
pub struct LogConfig {
    /// Whether debug mode is enabled
    pub debug: bool,
    /// Minimum log level
    pub level: Level,
    /// Output format
    pub format: Format,
    /// Whether colors are enabled
    pub color: bool,
    /// Whether to include timestamps
    pub timestamps: bool,
    /// Whether to include module path
    pub module_path: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            debug: false,
            level: Level::Info,
            format: Format::Json,
            color: false, // JSON output doesn't use colors
            timestamps: true,
            module_path: true,
        }
    }
}

impl LogConfig {
    /// Create config from environment variables.
    pub fn from_env() -> Self {
        let debug = env::var("ARMATURE_DEBUG")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(false);

        let level = env::var("ARMATURE_LOG_LEVEL")
            .ok()
            .and_then(|s| Level::parse(&s))
            .unwrap_or(if debug { Level::Debug } else { Level::Info });

        let format = env::var("ARMATURE_LOG_FORMAT")
            .ok()
            .and_then(|s| Format::parse(&s))
            .unwrap_or(Format::Json);

        let color = env::var("ARMATURE_LOG_COLOR")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(atty::is(atty::Stream::Stderr));

        let timestamps = env::var("ARMATURE_LOG_TIMESTAMPS")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(true);

        let module_path = env::var("ARMATURE_LOG_MODULE")
            .map(|v| v == "1" || v.to_lowercase() == "true")
            .unwrap_or(true);

        // Update global atomics
        DEBUG_ENABLED.store(debug, Ordering::SeqCst);
        LOG_LEVEL.store(level as u8, Ordering::SeqCst);

        Self {
            debug,
            level,
            format,
            color,
            timestamps,
            module_path,
        }
    }
}

/// Check if TTY (for color detection fallback).
mod atty {
    pub enum Stream {
        Stderr,
    }

    pub fn is(_stream: Stream) -> bool {
        // Simple check - assume color if not explicitly disabled
        std::env::var("NO_COLOR").is_err() && std::env::var("TERM").is_ok()
    }
}

// ============================================================================
// Public API
// ============================================================================

/// Initialize the logging system.
///
/// This is called automatically when first log macro is used,
/// but can be called explicitly for eager initialization.
pub fn init() {
    let config = Lazy::force(&CONFIG);
    // Initialize runtime atomics from config
    LOG_FORMAT.store(config.format as u8, Ordering::SeqCst);
    LOG_COLOR.store(config.color, Ordering::SeqCst);
    LOG_TIMESTAMPS.store(config.timestamps, Ordering::SeqCst);
    LOG_MODULE_PATH.store(config.module_path, Ordering::SeqCst);
}

/// Check if debug logging is enabled.
#[inline]
pub fn is_debug_enabled() -> bool {
    DEBUG_ENABLED.load(Ordering::Relaxed)
}

/// Check if a log level is enabled.
#[inline]
pub fn is_level_enabled(level: Level) -> bool {
    level as u8 >= LOG_LEVEL.load(Ordering::Relaxed)
}

/// Get current log level.
pub fn current_level() -> Level {
    match LOG_LEVEL.load(Ordering::Relaxed) {
        0 => Level::Trace,
        1 => Level::Debug,
        2 => Level::Info,
        3 => Level::Warn,
        4 => Level::Error,
        _ => Level::Off,
    }
}

/// Set log level at runtime.
pub fn set_level(level: Level) {
    LOG_LEVEL.store(level as u8, Ordering::SeqCst);
}

/// Enable or disable debug mode at runtime.
pub fn set_debug(enabled: bool) {
    DEBUG_ENABLED.store(enabled, Ordering::SeqCst);
    if enabled && current_level() > Level::Debug {
        set_level(Level::Debug);
    }
}

/// Get the global configuration.
pub fn config() -> &'static LogConfig {
    &CONFIG
}

// ============================================================================
// Runtime Configuration
// ============================================================================

use std::sync::atomic::AtomicU8 as AtomicFormat;

/// Global format setting (can be changed at runtime).
static LOG_FORMAT: AtomicFormat = AtomicFormat::new(Format::Json as u8);

/// Global color setting.
static LOG_COLOR: AtomicBool = AtomicBool::new(false);

/// Global timestamps setting.
static LOG_TIMESTAMPS: AtomicBool = AtomicBool::new(true);

/// Global module path setting.
static LOG_MODULE_PATH: AtomicBool = AtomicBool::new(true);

/// Get the current log format.
pub fn current_format() -> Format {
    match LOG_FORMAT.load(Ordering::Relaxed) {
        0 => Format::Pretty,
        1 => Format::Compact,
        _ => Format::Json,
    }
}

/// Set log format at runtime.
///
/// # Example
///
/// ```rust
/// use armature_log::{set_format, Format};
///
/// // Switch to pretty format for development
/// set_format(Format::Pretty);
///
/// // Switch back to JSON for production
/// set_format(Format::Json);
/// ```
pub fn set_format(format: Format) {
    LOG_FORMAT.store(format as u8, Ordering::SeqCst);
    // Also update color based on format
    if format == Format::Pretty {
        LOG_COLOR.store(atty::is(atty::Stream::Stderr), Ordering::SeqCst);
    } else if format == Format::Json {
        LOG_COLOR.store(false, Ordering::SeqCst);
    }
}

/// Set whether colors are enabled.
pub fn set_color(enabled: bool) {
    LOG_COLOR.store(enabled, Ordering::SeqCst);
}

/// Set whether timestamps are included.
pub fn set_timestamps(enabled: bool) {
    LOG_TIMESTAMPS.store(enabled, Ordering::SeqCst);
}

/// Set whether module path is included.
pub fn set_module_path(enabled: bool) {
    LOG_MODULE_PATH.store(enabled, Ordering::SeqCst);
}

/// Configuration builder for fluent API.
///
/// # Example
///
/// ```rust
/// use armature_log::{configure, Format, Level};
///
/// configure()
///     .format(Format::Pretty)
///     .level(Level::Debug)
///     .color(true)
///     .timestamps(true)
///     .apply();
/// ```
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    format: Option<Format>,
    level: Option<Level>,
    color: Option<bool>,
    timestamps: Option<bool>,
    module_path: Option<bool>,
    debug: Option<bool>,
}

impl ConfigBuilder {
    /// Create a new configuration builder.
    pub fn new() -> Self {
        Self {
            format: None,
            level: None,
            color: None,
            timestamps: None,
            module_path: None,
            debug: None,
        }
    }

    /// Set the output format.
    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    /// Set the log level.
    pub fn level(mut self, level: Level) -> Self {
        self.level = Some(level);
        self
    }

    /// Enable or disable colors.
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = Some(enabled);
        self
    }

    /// Enable or disable timestamps.
    pub fn timestamps(mut self, enabled: bool) -> Self {
        self.timestamps = Some(enabled);
        self
    }

    /// Enable or disable module path in output.
    pub fn module_path(mut self, enabled: bool) -> Self {
        self.module_path = Some(enabled);
        self
    }

    /// Enable or disable debug mode.
    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug = Some(enabled);
        self
    }

    /// Apply the configuration.
    pub fn apply(self) {
        if let Some(format) = self.format {
            set_format(format);
        }
        if let Some(level) = self.level {
            set_level(level);
        }
        if let Some(color) = self.color {
            set_color(color);
        }
        if let Some(timestamps) = self.timestamps {
            set_timestamps(timestamps);
        }
        if let Some(module_path) = self.module_path {
            set_module_path(module_path);
        }
        if let Some(debug) = self.debug {
            set_debug(debug);
        }
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a configuration builder.
///
/// # Example
///
/// ```rust
/// use armature_log::{configure, Format, Level};
///
/// // Development config
/// configure()
///     .format(Format::Pretty)
///     .level(Level::Debug)
///     .color(true)
///     .apply();
/// ```
pub fn configure() -> ConfigBuilder {
    ConfigBuilder::new()
}

/// Apply development preset: Pretty format, Debug level, colors enabled.
///
/// # Example
///
/// ```rust
/// armature_log::preset_development();
/// ```
pub fn preset_development() {
    configure()
        .format(Format::Pretty)
        .level(Level::Debug)
        .color(true)
        .timestamps(true)
        .module_path(true)
        .debug(true)
        .apply();
}

/// Apply production preset: JSON format, Info level, no colors.
///
/// # Example
///
/// ```rust
/// armature_log::preset_production();
/// ```
pub fn preset_production() {
    configure()
        .format(Format::Json)
        .level(Level::Info)
        .color(false)
        .timestamps(true)
        .module_path(true)
        .debug(false)
        .apply();
}

/// Apply quiet preset: JSON format, Warn level only.
pub fn preset_quiet() {
    configure()
        .format(Format::Json)
        .level(Level::Warn)
        .color(false)
        .apply();
}

// ============================================================================
// Log Output
// ============================================================================

/// Log a message with the given level.
#[doc(hidden)]
pub fn log(level: Level, target: &str, message: &str) {
    if !is_level_enabled(level) {
        return;
    }

    // Use runtime-configurable format instead of static config
    let format = current_format();
    let color = LOG_COLOR.load(Ordering::Relaxed);
    let timestamps = LOG_TIMESTAMPS.load(Ordering::Relaxed);
    let module_path = LOG_MODULE_PATH.load(Ordering::Relaxed);

    match format {
        Format::Pretty => log_pretty_runtime(level, target, message, color, timestamps, module_path),
        Format::Compact => log_compact_runtime(level, target, message, timestamps, module_path),
        Format::Json => log_json(level, target, message),
    }
}

#[allow(dead_code)]
fn log_pretty(level: Level, target: &str, message: &str, config: &LogConfig) {
    log_pretty_runtime(level, target, message, config.color, config.timestamps, config.module_path);
}

#[allow(dead_code)]
fn log_compact(level: Level, target: &str, message: &str, config: &LogConfig) {
    log_compact_runtime(level, target, message, config.timestamps, config.module_path);
}

// Runtime-configurable versions

fn log_pretty_runtime(
    level: Level,
    target: &str,
    message: &str,
    color: bool,
    timestamps: bool,
    module_path: bool,
) {
    let mut stderr = std::io::stderr().lock();

    // Timestamp
    if timestamps {
        let now = chrono::Local::now();
        let _ = write!(stderr, "{} ", now.format("%Y-%m-%d %H:%M:%S%.3f"));
    }

    // Level
    #[cfg(feature = "color")]
    if color {
        let _ = write!(stderr, "{:5} ", level.colored());
    } else {
        let _ = write!(stderr, "{:5} ", level.as_str());
    }

    #[cfg(not(feature = "color"))]
    {
        let _ = color; // suppress warning
        let _ = write!(stderr, "{:5} ", level.as_str());
    }

    // Target
    if module_path && !target.is_empty() {
        #[cfg(feature = "color")]
        if color {
            use colored::Colorize;
            let _ = write!(stderr, "{} ", target.dimmed());
        } else {
            let _ = write!(stderr, "[{}] ", target);
        }

        #[cfg(not(feature = "color"))]
        let _ = write!(stderr, "[{}] ", target);
    }

    // Message
    let _ = writeln!(stderr, "{}", message);
}

fn log_compact_runtime(
    level: Level,
    target: &str,
    message: &str,
    timestamps: bool,
    module_path: bool,
) {
    let mut stderr = std::io::stderr().lock();

    if timestamps {
        let now = chrono::Local::now();
        let _ = write!(stderr, "{} ", now.format("%H:%M:%S"));
    }

    let _ = write!(stderr, "{} ", level.as_str().chars().next().unwrap_or('?'));

    if module_path && !target.is_empty() {
        let _ = write!(stderr, "{}: ", target);
    }

    let _ = writeln!(stderr, "{}", message);
}

#[cfg(feature = "json")]
fn log_json(level: Level, target: &str, message: &str) {
    use serde::Serialize;

    #[derive(Serialize)]
    struct LogEntry<'a> {
        timestamp: String,
        level: &'a str,
        target: &'a str,
        message: &'a str,
    }

    let entry = LogEntry {
        timestamp: chrono::Utc::now().to_rfc3339(),
        level: level.as_str(),
        target,
        message,
    };

    if let Ok(json) = serde_json::to_string(&entry) {
        eprintln!("{}", json);
    }
}

#[cfg(not(feature = "json"))]
fn log_json(level: Level, target: &str, message: &str) {
    // Fallback without serde - manually escape JSON strings
    let timestamp = chrono::Utc::now().to_rfc3339();
    eprintln!(
        r#"{{"timestamp":"{}","level":"{}","target":"{}","message":"{}"}}"#,
        timestamp,
        level.as_str(),
        escape_json(target),
        escape_json(message)
    );
}

#[cfg(not(feature = "json"))]
fn escape_json(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

// ============================================================================
// Macros
// ============================================================================

/// Log a trace message.
///
/// Only enabled when `ARMATURE_DEBUG=1` or `ARMATURE_LOG_LEVEL=trace`.
#[macro_export]
macro_rules! trace {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Trace) {
            $crate::log($crate::Level::Trace, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Trace) {
            $crate::log($crate::Level::Trace, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log a debug message.
///
/// Only enabled when `ARMATURE_DEBUG=1` or `ARMATURE_LOG_LEVEL=debug`.
///
/// # Example
///
/// ```rust
/// use armature_log::debug;
///
/// debug!("Processing request");
/// let username = "alice";
/// debug!("User {} logged in", username);
/// let path = "/api/users";
/// debug!(target: "armature::router", "Matching route: {}", path);
/// ```
#[macro_export]
macro_rules! debug {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_debug_enabled() || $crate::is_level_enabled($crate::Level::Debug) {
            $crate::log($crate::Level::Debug, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_debug_enabled() || $crate::is_level_enabled($crate::Level::Debug) {
            $crate::log($crate::Level::Debug, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log an info message.
#[macro_export]
macro_rules! info {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Info) {
            $crate::log($crate::Level::Info, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Info) {
            $crate::log($crate::Level::Info, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log a warning message.
#[macro_export]
macro_rules! warn {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Warn) {
            $crate::log($crate::Level::Warn, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Warn) {
            $crate::log($crate::Level::Warn, module_path!(), &format!($($arg)+));
        }
    };
}

/// Log an error message.
#[macro_export]
macro_rules! error {
    (target: $target:expr, $($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Error) {
            $crate::log($crate::Level::Error, $target, &format!($($arg)+));
        }
    };
    ($($arg:tt)+) => {
        if $crate::is_level_enabled($crate::Level::Error) {
            $crate::log($crate::Level::Error, module_path!(), &format!($($arg)+));
        }
    };
}

// ============================================================================
// Tracing Integration
// ============================================================================

#[cfg(feature = "tracing")]
pub mod tracing_compat {
    //! Tracing compatibility layer.
    //!
    //! When the `tracing` feature is enabled, this module provides
    //! a subscriber that respects `ARMATURE_DEBUG`.

    use super::*;

    /// Create a tracing subscriber that respects Armature config.
    pub fn subscriber() -> impl tracing::Subscriber {
        use tracing_subscriber::prelude::*;
        use tracing_subscriber::{fmt, EnvFilter};

        let config = config();
        let level = match config.level {
            Level::Trace => "trace",
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
            Level::Off => "off",
        };

        let filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new(level));

        tracing_subscriber::registry()
            .with(filter)
            .with(fmt::layer().with_ansi(config.color))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_ordering() {
        assert!(Level::Trace < Level::Debug);
        assert!(Level::Debug < Level::Info);
        assert!(Level::Info < Level::Warn);
        assert!(Level::Warn < Level::Error);
        assert!(Level::Error < Level::Off);
    }

    #[test]
    fn test_level_parse() {
        assert_eq!(Level::parse("debug"), Some(Level::Debug));
        assert_eq!(Level::parse("DEBUG"), Some(Level::Debug));
        assert_eq!(Level::parse("warn"), Some(Level::Warn));
        assert_eq!(Level::parse("warning"), Some(Level::Warn));
        assert_eq!(Level::parse("invalid"), None);
    }

    #[test]
    fn test_format_parse() {
        assert_eq!(Format::parse("pretty"), Some(Format::Pretty));
        assert_eq!(Format::parse("compact"), Some(Format::Compact));
        assert_eq!(Format::parse("json"), Some(Format::Json));
        assert_eq!(Format::parse("invalid"), None);
    }

    #[test]
    fn test_set_level() {
        let original = current_level();

        set_level(Level::Error);
        assert_eq!(current_level(), Level::Error);

        set_level(Level::Debug);
        assert_eq!(current_level(), Level::Debug);

        set_level(original);
    }

    #[test]
    fn test_debug_flag() {
        let original = is_debug_enabled();

        set_debug(true);
        assert!(is_debug_enabled());

        set_debug(false);
        assert!(!is_debug_enabled());

        set_debug(original);
    }

    #[test]
    fn test_macros_compile() {
        // Just verify macros compile correctly
        trace!("trace message");
        debug!("debug message");
        info!("info message");
        warn!("warn message");
        error!("error message");

        trace!(target: "test", "with target");
        debug!(target: "test", "with target");
        info!(target: "test", "with target");
        warn!(target: "test", "with target");
        error!(target: "test", "with target");

        let x = 42;
        debug!("formatted: {}", x);
    }
}


use clap::{Parser, ValueEnum};
use std::error::Error;
use tracing::Level;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::util::TryInitError;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, serde::Serialize, serde::Deserialize)]
pub enum LogFormat {
	Compact,
	Pretty,
	JSON,
	// None,
}

#[derive(Parser, Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct LogArgs {
	/// Verbosity logging level
	#[arg(short, long, default_value_t = Level::INFO)]
	#[serde(with = "with_tracing_level")]
	pub verbosity: Level,

	/// Log format type on stderr.
	#[arg(value_enum, long, default_value_t = LogFormat::Compact)]
	pub log_format: LogFormat,

	/// Log filter format string
	#[arg(long, env = "LOG_FILTER", default_value = "")]
	pub log_filter: String,
}

impl Default for LogArgs {
	fn default() -> Self {
		LogArgs {
			verbosity: Level::INFO,
			log_format: LogFormat::Compact,
			log_filter: String::new(),
		}
	}
}

mod with_tracing_level {
	use serde::Deserialize;

	// I have no control over the signature for this function clippy, hush
	#[allow(clippy::trivially_copy_pass_by_ref)]
	pub fn serialize<S: serde::Serializer>(level: &tracing::Level, s: S) -> Result<S::Ok, S::Error> {
		s.serialize_str(&level.to_string())
	}

	pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<tracing::Level, D::Error> {
		String::deserialize(d)?
			.parse()
			.map_err(|e| serde::de::Error::custom(format!("{e:?} : Valid values are: error, warn, info, debug, trace")))
	}
}

#[derive(Debug)]
pub enum LoggerError {
	TracingSubscriberInitError(TryInitError),
}

impl std::fmt::Display for LoggerError {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			LoggerError::TracingSubscriberInitError(e) => write!(f, "Tracing subscriber init error: {e}"),
		}
	}
}

impl Error for LoggerError {
	fn source(&self) -> Option<&(dyn Error + 'static)> {
		match self {
			LoggerError::TracingSubscriberInitError(e) => Some(e),
		}
	}
}

impl From<TryInitError> for LoggerError {
	fn from(e: TryInitError) -> Self {
		LoggerError::TracingSubscriberInitError(e)
	}
}

/// Initialize a general logging system built with `tracing` and `tracing-subscriber`.
///
/// It includes a convenient clap argument parser for the logging system to allow it
/// to be configured from the command line.
///
/// # Errors
///
/// Returns an error if it fails to subscribe the logging system.
pub fn init_logger(args: &LogArgs) -> Result<(), LoggerError> {
	let builder = FmtSubscriber::builder()
		.with_writer(std::io::stderr)
		.with_max_level(args.verbosity)
		.with_level(true)
		//.with_span_events(FmtSpan::FULL)
		.with_span_events(
			// Keep the individual choices for ease of modification
			#[allow(clippy::match_same_arms)]
			match args.verbosity {
				Level::ERROR => FmtSpan::NEW | FmtSpan::CLOSE,
				Level::WARN => FmtSpan::NEW | FmtSpan::CLOSE,
				Level::INFO => FmtSpan::NEW | FmtSpan::CLOSE,
				Level::DEBUG => FmtSpan::FULL,
				Level::TRACE => FmtSpan::FULL,
			},
		)
		.with_target(true)
		.with_ansi(true)
		.with_env_filter(EnvFilter::try_new(&args.log_filter).unwrap_or_else(|err| {
			eprintln!("Invalid log filter: {err}");
			EnvFilter::new(&args.log_filter)
		}));
	match args.log_format {
		LogFormat::Compact => builder.compact().finish().try_init()?,
		LogFormat::Pretty => builder.pretty().finish().try_init()?,
		LogFormat::JSON => builder.json().finish().try_init()?,
		// LogFormat::None => builder.finish().try_init()?,
	}
	tracing::info!("Initialized logging system at level: {}", args.verbosity);
	Ok(())
}

use colored::*;
use tracing::level_filters::LevelFilter;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::filter::filter_fn;
use tracing_subscriber::fmt::{self, FormatEvent, FormatFields};
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{EnvFilter, prelude::*};

/// Custom formatter used only for the "BANNER" target.
struct BannerDotFormatter;

/// Small visitor that extracts the optional `tag = "..."`
#[derive(Default)]
struct TagVisitor {
  tag: Option<String>,
}

impl tracing::field::Visit for TagVisitor {
  fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
    if field.name() == "tag" {
      self.tag = Some(value.to_string());
    }
  }

  fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
    if field.name() == "tag" {
      self.tag = Some(format!("{value:?}"));
    }
  }
}

impl<S, N> FormatEvent<S, N> for BannerDotFormatter
where
  S: Subscriber + for<'a> LookupSpan<'a>,
  N: for<'a> FormatFields<'a> + 'static,
{
  fn format_event(
    &self,
    ctx: &fmt::FmtContext<'_, S, N>,
    mut writer: fmt::format::Writer<'_>,
    event: &Event<'_>,
  ) -> std::fmt::Result {
    let meta = event.metadata();

    // Dot color by level.
    let dot = match *meta.level() {
      Level::ERROR => "●".red(),
      Level::WARN => "●".yellow(),
      _ => "●".green(),
    };

    // Try to extract `tag=...` if provided.
    let mut v = TagVisitor::default();
    event.record(&mut v);

    // Layout:
    // - if tag exists: "●      <tag> » <msg>"
    // - else:          "● » <msg>"
    if let Some(tag) = v.tag.as_deref() {
      write!(writer, "{} {:>10} {} ", dot, tag.bright_black().bold(), "»".bright_black())?;
    } else {
      write!(writer, "{} {} ", dot, "»".bright_black())?;
    }

    // Print the event fields/message.
    ctx.format_fields(writer.by_ref(), event)?;
    writeln!(writer)
  }
}

/// Map CLI verbosity flags into a tracing level.
///
/// - `-q` (quiet) forces ERROR
/// - default is WARN (nice "banner mode")
/// - `-v` => INFO, `-vv` => DEBUG, `-vvv` => TRACE
fn parse_level(verbose: u8, quiet: bool) -> LevelFilter {
  if quiet {
    return LevelFilter::ERROR;
  }
  match verbose {
    0 => LevelFilter::WARN,
    1 => LevelFilter::INFO,
    2 => LevelFilter::DEBUG,
    _ => LevelFilter::TRACE,
  }
}

/// Initialize tracing with two layers:
/// Filter precedence:
/// - If `RUST_LOG` is set, it wins.
/// - Otherwise we derive the level from `verbose`/`quiet`.
pub fn setup_logging(verbose: u8, quiet: bool) {
  // Prefer RUST_LOG if present; otherwise derive from -v/-q.
  let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
    let level = parse_level(verbose, quiet);
    EnvFilter::new(level.to_string())
  });

  let level = filter.max_level_hint().unwrap_or(LevelFilter::INFO);

  // Banner mode = clean startup output:
  // - dot-format only for target "BANNER"
  // - normal logs still flow (subject to `filter`)
  //
  // Quiet disables banner completely.
  let banner_mode = !quiet && level <= LevelFilter::WARN;

  // Normal app logs; hide "BANNER" events when banner mode is active
  // (so they don't show twice).
  let app_layer = fmt::layer()
    .with_writer(std::io::stdout)
    .with_filter(filter.clone())
    .with_filter(filter_fn(move |meta| {
      // TODO: Validate whether the banner should not be displayed in banner_mode, but be displayed as a normal record.
      meta.target() != "BANNER" || !banner_mode
    }));

  let registry = tracing_subscriber::registry().with(app_layer);

  if banner_mode {
    let banner_layer = fmt::layer()
      .with_writer(std::io::stdout)
      .event_format(BannerDotFormatter)
      .with_filter(filter_fn(|meta| meta.target() == "BANNER"));

    registry.with(banner_layer).init();
  } else {
    registry.init();
  }
}

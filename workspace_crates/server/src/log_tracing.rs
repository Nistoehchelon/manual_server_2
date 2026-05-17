use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::RollingFileAppender;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use tracing_subscriber::fmt;

// purpose: fix duplication when using multiple formatters (in this case fileout and stdout)
// found in https://github.com/tokio-rs/tracing/issues/1372#issuecomment-2109411275
// Define two distinct types even if they do the same thing
#[derive(Default)]
struct StdoutFields(fmt::format::DefaultFields);
#[derive(Default)]
struct FileFields(fmt::format::DefaultFields);

// Implement the trait for both (boilerplate)
impl<'writer> fmt::FormatFields<'writer> for StdoutFields {
    fn format_fields<R>(&self, w: fmt::format::Writer<'writer>, f: R) -> std::fmt::Result
    where
        R: tracing_subscriber::field::RecordFields,
    {
        self.0.format_fields(w, f)
    }
}
impl<'writer> fmt::FormatFields<'writer> for FileFields {
    fn format_fields<R>(&self, w: fmt::format::Writer<'writer>, f: R) -> std::fmt::Result
    where
        R: tracing_subscriber::field::RecordFields,
    {
        self.0.format_fields(w, f)
    }
}

pub fn init_tracing(enable_stdout: bool, file_appender: RollingFileAppender) -> WorkerGuard {
    let (file_writer, _guard) = tracing_appender::non_blocking(file_appender);

    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(file_writer)
        .with_ansi(false)
        .with_target(true)
        .with_line_number(true)
        .fmt_fields(FileFields::default()); // important for files

    // Optional stdout layer
    let stdout_layer = tracing_subscriber::fmt::layer()
        .with_target(true)
        .with_line_number(true)
        .fmt_fields(StdoutFields::default());

    let filter = EnvFilter::from_default_env();

    let subscriber = tracing_subscriber::registry().with(filter).with(file_layer);

    if enable_stdout {
        subscriber.with(stdout_layer).init();
    } else {
        subscriber.init();
    }
    _guard
}

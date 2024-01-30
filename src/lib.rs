use crate::initialize::{build_metrics_provider, init_logging, init_propagator, init_tracing};
use time::macros::offset;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod initialize;

#[derive(Default)]
pub struct NewRelicSubscriberInitializer {
    newrelic_otlp_endpoint: Option<String>,
    newrelic_license_key: Option<String>,
    newrelic_service_name: Option<String>,
}

impl NewRelicSubscriberInitializer {
    pub fn newrelic_otlp_endpoint(mut self, newrelic_otlp_endpoint: &str) -> Self {
        self.newrelic_otlp_endpoint = Some(newrelic_otlp_endpoint.to_string());
        self
    }

    pub fn newrelic_license_key(mut self, newrelic_license_key: &str) -> Self {
        self.newrelic_license_key = Some(newrelic_license_key.to_string());
        self
    }

    pub fn newrelic_service_name(mut self, newrelic_service_name: &str) -> Self {
        self.newrelic_service_name = Some(newrelic_service_name.to_string());
        self
    }

    pub fn init(self) {
        let newrelic_otlp_endpoint = self.newrelic_otlp_endpoint.unwrap_or_default();
        let newrelic_license_key = self.newrelic_license_key.unwrap_or_default();
        let newrelic_service_name = self.newrelic_service_name.unwrap_or_default();
        init_propagator();

        let tracer = init_tracing(
            &newrelic_otlp_endpoint,
            &newrelic_license_key,
            &newrelic_service_name,
        );

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
                offset!(+09:00:00),
                time::format_description::well_known::Rfc3339,
            ))
            .with_target(true);
        let env_filter =
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());
        let otel_trace_layer = tracing_opentelemetry::layer()
            .with_error_records_to_exceptions(true)
            .with_error_fields_to_exceptions(true)
            .with_error_events_to_status(true)
            .with_error_events_to_exceptions(true)
            .with_location(true)
            .with_tracer(tracer);
        let otel_metrics_layer = tracing_opentelemetry::MetricsLayer::new(build_metrics_provider(
            &newrelic_otlp_endpoint,
            &newrelic_license_key,
            &newrelic_service_name,
        ));
        init_logging(
            &newrelic_otlp_endpoint,
            &newrelic_license_key,
            &newrelic_service_name,
        );
        let logger_provider = opentelemetry::global::logger_provider();
        let otel_logs_layer =
            opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                &logger_provider,
            );

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(env_filter)
            .with(otel_trace_layer)
            .with(otel_metrics_layer)
            .with(otel_logs_layer)
            .init();
    }
}

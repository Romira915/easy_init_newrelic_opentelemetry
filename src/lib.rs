//! # New Relic OpenTelemetry Subscriber
//! This crate provides a subscriber for OpenTelemetry that sends spans and metrics to New Relic.
//!
//! ## Example
//! ```rust
//! use easy_init_newrelic_opentelemetry::NewRelicSubscriberInitializer;
//! use time::macros::offset;
//!
//! NewRelicSubscriberInitializer::default()
//!             .newrelic_otlp_endpoint("http://localhost:4317")
//!             .newrelic_license_key("1234567890abcdef1234567890abcdef12345678")
//!             .newrelic_service_name("test-service")
//!             .host_name("test-host")
//!             .timestamps_offset(offset!(+00:00:00));
//!             // init();
//! ```

use crate::initialize::{
    init_logger_provider, init_metrics, init_process_metrics, init_tracer_provider,
};
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::ExporterBuildError;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use time::macros::offset;
use time::UtcOffset;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

mod initialize;

const NEWRELIC_OTLP_ENDPOINT: &str = "https://otlp.nr-data.net";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to build OTLP exporter: {0}")]
    ExporterBuild(#[from] ExporterBuildError),
}

/// A guard that keeps telemetry providers alive and shuts them down on drop.
///
/// Hold this value in your `main` function to ensure telemetry data is flushed
/// when the application exits.
pub struct TelemetryGuard {
    tracer_provider: Option<SdkTracerProvider>,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
}

impl Drop for TelemetryGuard {
    fn drop(&mut self) {
        if let Some(provider) = &self.tracer_provider {
            if let Err(e) = provider.shutdown() {
                eprintln!("Failed to shutdown tracer provider: {e}");
            }
        }
        if let Some(provider) = &self.meter_provider {
            if let Err(e) = provider.shutdown() {
                eprintln!("Failed to shutdown meter provider: {e}");
            }
        }
        if let Some(provider) = &self.logger_provider {
            if let Err(e) = provider.shutdown() {
                eprintln!("Failed to shutdown logger provider: {e}");
            }
        }
    }
}

#[derive(Default)]
pub struct NewRelicSubscriberInitializer {
    newrelic_otlp_endpoint: Option<String>,
    newrelic_license_key: Option<String>,
    newrelic_service_name: Option<String>,
    host_name: Option<String>,
    timestamps_offset: Option<UtcOffset>,
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

    pub fn host_name(mut self, host_name: &str) -> Self {
        self.host_name = Some(host_name.to_string());
        self
    }

    pub fn timestamps_offset(mut self, timestamps_offset: UtcOffset) -> Self {
        self.timestamps_offset = Some(timestamps_offset);
        self
    }

    /// Initialize OpenTelemetry tracing, metrics, and logging exporters for New Relic.
    ///
    /// Returns a [`TelemetryGuard`] that must be held alive for the lifetime of the application.
    /// When dropped, it gracefully shuts down all telemetry providers, flushing any pending data.
    ///
    /// # Environment variable fallback
    ///
    /// Builder values take precedence. If not set, the following environment variables are used:
    /// - `NEW_RELIC_LICENSE_KEY` → license key
    /// - `OTEL_SERVICE_NAME` → service name
    /// - `OTEL_EXPORTER_OTLP_ENDPOINT` → OTLP endpoint
    ///
    /// If the license key is not set by either the builder or the environment variable,
    /// OTLP exporters are **skipped** and only the local fmt layer is enabled.
    pub fn init(self) -> Result<TelemetryGuard, Error> {
        let newrelic_otlp_endpoint = self
            .newrelic_otlp_endpoint
            .or_else(|| std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok())
            .unwrap_or_else(|| NEWRELIC_OTLP_ENDPOINT.to_string());
        let newrelic_license_key = self
            .newrelic_license_key
            .or_else(|| std::env::var("NEW_RELIC_LICENSE_KEY").ok());
        let newrelic_service_name = self
            .newrelic_service_name
            .or_else(|| std::env::var("OTEL_SERVICE_NAME").ok())
            .unwrap_or_default();
        let host_name = self.host_name.unwrap_or_default();
        let timestamps_offset = self.timestamps_offset.unwrap_or(offset!(+00:00:00));

        let fmt_layer = tracing_subscriber::fmt::layer()
            .with_ansi(true)
            .with_file(true)
            .with_line_number(true)
            .with_target(true)
            .with_timer(tracing_subscriber::fmt::time::OffsetTime::new(
                timestamps_offset,
                time::format_description::well_known::Rfc3339,
            ));
        let env_filter =
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into());

        let (
            otel_trace_layer,
            otel_metrics_layer,
            otel_logs_layer,
            tracer_provider,
            meter_provider,
            logger_provider,
        ) = if let Some(license_key) = &newrelic_license_key {
            let tracer_provider = init_tracer_provider(
                &newrelic_otlp_endpoint,
                license_key,
                &newrelic_service_name,
                &host_name,
            )?;
            opentelemetry::global::set_tracer_provider(tracer_provider.clone());
            let tracer = tracer_provider.tracer(newrelic_service_name.clone());

            let otel_trace_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer)
                .with_error_records_to_exceptions(true)
                .with_error_fields_to_exceptions(true)
                .with_error_events_to_status(true)
                .with_error_events_to_exceptions(true)
                .with_location(true);

            let meter_provider = init_metrics(
                &newrelic_otlp_endpoint,
                license_key,
                &newrelic_service_name,
                &host_name,
            )?;
            opentelemetry::global::set_meter_provider(meter_provider.clone());
            init_process_metrics(&meter_provider);
            let otel_metrics_layer =
                tracing_opentelemetry::MetricsLayer::new(meter_provider.clone());

            let logger_provider = init_logger_provider(
                &newrelic_otlp_endpoint,
                license_key,
                &newrelic_service_name,
                &host_name,
            )?;
            let otel_logs_layer =
                opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge::new(
                    &logger_provider,
                );

            (
                Some(otel_trace_layer),
                Some(otel_metrics_layer),
                Some(otel_logs_layer),
                Some(tracer_provider),
                Some(meter_provider),
                Some(logger_provider),
            )
        } else {
            (None, None, None, None, None, None)
        };

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(env_filter)
            .with(otel_trace_layer)
            .with(otel_metrics_layer)
            .with(otel_logs_layer)
            .init();

        if newrelic_license_key.is_none() {
            tracing::warn!(
                "NEW_RELIC_LICENSE_KEY is not set. OTLP exporters are disabled."
            );
        }

        Ok(TelemetryGuard {
            tracer_provider,
            meter_provider,
            logger_provider,
        })
    }
}

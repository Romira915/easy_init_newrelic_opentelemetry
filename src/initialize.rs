use opentelemetry::metrics::MetricsError;
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{HttpExporterBuilder, Protocol, WithExportConfig};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::reader::DefaultTemporalitySelector;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{trace, Resource};
use std::collections::HashMap;

fn http_exporter() -> HttpExporterBuilder {
    opentelemetry_otlp::new_exporter()
        .http()
        .with_protocol(Protocol::HttpJson)
}

pub(crate) fn init_logger_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> Result<LoggerProvider, opentelemetry::logs::LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            http_exporter()
                .with_endpoint(format!("{}/v1/logs", new_relic_otlp_endpoint))
                .with_headers(HashMap::from([(
                    "api-key".into(),
                    new_relic_license_key.into(),
                )])),
        )
        .with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                new_relic_service_name.to_string(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::HOST_NAME,
                host_name.to_string(),
            ),
        ]))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

pub(crate) fn init_tracer_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> Result<TracerProvider, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            http_exporter()
                .with_endpoint(format!("{}/v1/traces", new_relic_otlp_endpoint))
                .with_headers(HashMap::from([(
                    "api-key".into(),
                    new_relic_license_key.into(),
                )])),
        )
        .with_trace_config(trace::Config::default().with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                new_relic_service_name.to_string(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::HOST_NAME,
                host_name.to_string(),
            ),
        ])))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

pub(crate) fn init_metrics(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            http_exporter()
                .with_endpoint(format!("{}/v1/metrics", new_relic_otlp_endpoint))
                .with_headers(HashMap::from([(
                    "api-key".into(),
                    new_relic_license_key.into(),
                )])),
        )
        .with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                new_relic_service_name.to_string(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::HOST_NAME,
                host_name.to_string(),
            ),
        ]))
        .with_period(std::time::Duration::from_secs(3))
        .with_timeout(std::time::Duration::from_secs(10))
        // .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()
}

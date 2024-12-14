use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{
    LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::MetricError;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{runtime, Resource};
use std::collections::HashMap;

fn resource(new_relic_service_name: &str, host_name: &str) -> Resource {
    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            new_relic_service_name.to_string(),
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::HOST_NAME,
            host_name.to_string(),
        ),
    ])
}

pub(crate) fn init_logger_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> Result<LoggerProvider, opentelemetry_sdk::logs::LogError> {
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/logs", new_relic_otlp_endpoint))
        .with_headers(HashMap::from([(
            "api-key".into(),
            new_relic_license_key.into(),
        )]))
        .with_protocol(Protocol::HttpJson)
        .build()?;

    Ok(LoggerProvider::builder()
        .with_resource(resource(new_relic_service_name, host_name))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build())
}

pub(crate) fn init_tracer_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> Result<TracerProvider, TraceError> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", new_relic_otlp_endpoint))
        .with_headers(HashMap::from([(
            "api-key".into(),
            new_relic_license_key.into(),
        )]))
        .with_protocol(Protocol::HttpJson)
        .build()?;

    Ok(TracerProvider::builder()
        .with_resource(resource(new_relic_service_name, host_name))
        .with_batch_exporter(exporter, runtime::Tokio)
        .build())
}

pub(crate) fn init_metrics(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricError> {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/metrics", new_relic_otlp_endpoint))
        .with_headers(HashMap::from([(
            "api-key".into(),
            new_relic_license_key.into(),
        )]))
        .with_protocol(Protocol::HttpJson)
        .build()?;

    let reader =
        opentelemetry_sdk::metrics::PeriodicReader::builder(exporter, runtime::Tokio).build();

    Ok(opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource(new_relic_service_name, host_name))
        .build())
}

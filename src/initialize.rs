use opentelemetry::KeyValue;
use opentelemetry_otlp::{
    LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::collections::HashMap;

fn resource(new_relic_service_name: &str, host_name: &str) -> Resource {
    Resource::builder()
        .with_attributes(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                new_relic_service_name.to_string(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::HOST_NAME,
                host_name.to_string(),
            ),
        ])
        .build()
}

pub(crate) fn init_logger_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> anyhow::Result<SdkLoggerProvider> {
    let exporter = LogExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/logs", new_relic_otlp_endpoint))
        .with_headers(HashMap::from([(
            "api-key".into(),
            new_relic_license_key.into(),
        )]))
        .with_protocol(Protocol::HttpJson)
        .build()?;

    Ok(SdkLoggerProvider::builder()
        .with_resource(resource(new_relic_service_name, host_name))
        .with_batch_exporter(exporter)
        .build())
}

pub(crate) fn init_tracer_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> anyhow::Result<SdkTracerProvider> {
    let exporter = SpanExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/traces", new_relic_otlp_endpoint))
        .with_headers(HashMap::from([(
            "api-key".into(),
            new_relic_license_key.into(),
        )]))
        .with_protocol(Protocol::HttpJson)
        .build()?;

    Ok(SdkTracerProvider::builder()
        .with_resource(resource(new_relic_service_name, host_name))
        .with_batch_exporter(exporter)
        .build())
}

pub(crate) fn init_metrics(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
    host_name: &str,
) -> anyhow::Result<opentelemetry_sdk::metrics::SdkMeterProvider> {
    let exporter = MetricExporter::builder()
        .with_http()
        .with_endpoint(format!("{}/v1/metrics", new_relic_otlp_endpoint))
        .with_headers(HashMap::from([(
            "api-key".into(),
            new_relic_license_key.into(),
        )]))
        .with_protocol(Protocol::HttpJson)
        .build()?;

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter).build();

    Ok(opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(resource(new_relic_service_name, host_name))
        .build())
}

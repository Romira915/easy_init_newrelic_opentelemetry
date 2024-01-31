use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::Logger;
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::metrics::MeterProvider;
use opentelemetry_sdk::propagation::{
    BaggagePropagator, TextMapCompositePropagator, TraceContextPropagator,
};
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::{trace, Resource};
use std::collections::HashMap;

fn propagator_from_string(
    v: &str,
) -> Result<Option<Box<dyn TextMapPropagator + Send + Sync>>, TraceError> {
    match v {
        "tracecontext" => Ok(Some(Box::new(TraceContextPropagator::new()))),
        "baggage" => Ok(Some(Box::new(BaggagePropagator::new()))),
        _ => Ok(None),
    }
}

pub(crate) fn init_propagator() {
    let value_from_env =
        std::env::var("OTEL_PROPAGATORS").unwrap_or_else(|_| "tracecontext,baggage".to_string());
    let propagators: Vec<(Box<dyn TextMapPropagator + Send + Sync>, String)> = value_from_env
        .split(',')
        .map(|s| {
            let name = s.trim().to_lowercase();
            propagator_from_string(&name).map(|o| o.map(|b| (b, name)))
        })
        .collect::<Result<Vec<_>, _>>()
        .expect("Failed to create propagator.")
        .into_iter()
        .flatten()
        .collect();
    if !propagators.is_empty() {
        let (propagators_impl, propagators_name): (Vec<_>, Vec<_>) =
            propagators.into_iter().unzip();
        tracing::debug!(target: "otel::setup", OTEL_PROPAGATORS = propagators_name.join(","));
        let composite_propagator = TextMapCompositePropagator::new(propagators_impl);
        opentelemetry::global::set_text_map_propagator(composite_propagator);
    }
}

pub(crate) fn init_logging(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
) -> Logger {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(new_relic_otlp_endpoint)
                .with_headers(HashMap::from([(
                    "api-key".into(),
                    new_relic_license_key.into(),
                )])),
        )
        .with_log_config(
            opentelemetry_sdk::logs::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                new_relic_service_name.to_string(),
            )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to create logging controller.")
}

pub(crate) fn build_metrics_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
) -> MeterProvider {
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(new_relic_otlp_endpoint)
                .with_headers(HashMap::from([(
                    "api-key".into(),
                    new_relic_license_key.into(),
                )])),
        )
        .with_resource(Resource::new(vec![KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            new_relic_service_name.to_string(),
        )]))
        .with_period(std::time::Duration::from_secs(3))
        .with_timeout(std::time::Duration::from_secs(10))
        .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()
        .expect("Failed to create metrics provider.")
}

pub(crate) fn init_tracing(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    new_relic_service_name: &str,
) -> Tracer {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(new_relic_otlp_endpoint)
                .with_headers(HashMap::from([(
                    "api-key".into(),
                    new_relic_license_key.into(),
                )])),
        )
        .with_trace_config(
            trace::config().with_resource(Resource::new(vec![KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                new_relic_service_name.to_string(),
            )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to create tracer.")
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEWRELIC_OTLP_ENDPOINT: &str = "http://localhost:4317";
    const NEWRELIC_LICENSE_KEY: &str = "1234567890abcdef1234567890abcdef12345678";
    const NEWRELIC_SERVICE_NAME: &str = "test-service";

    #[tokio::test]
    async fn test_init_propagator() {
        init_propagator();
    }

    #[tokio::test]
    async fn test_init_logging() {
        init_logging(
            NEWRELIC_OTLP_ENDPOINT,
            NEWRELIC_LICENSE_KEY,
            NEWRELIC_SERVICE_NAME,
        );
    }

    #[tokio::test]
    async fn test_build_metrics_provider() {
        build_metrics_provider(
            NEWRELIC_OTLP_ENDPOINT,
            NEWRELIC_LICENSE_KEY,
            NEWRELIC_SERVICE_NAME,
        );
    }

    #[tokio::test]
    async fn test_init_tracing() {
        init_tracing(
            NEWRELIC_OTLP_ENDPOINT,
            NEWRELIC_LICENSE_KEY,
            NEWRELIC_SERVICE_NAME,
        );
    }
}

use opentelemetry::metrics::{AsyncInstrument, MeterProvider};
use opentelemetry::KeyValue;
use opentelemetry_otlp::{
    LogExporter, MetricExporter, Protocol, SpanExporter, WithExportConfig, WithHttpConfig,
};
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::SdkTracerProvider;
use opentelemetry_sdk::Resource;
use std::collections::HashMap;
use std::time::Instant;

pub(crate) fn resource(new_relic_service_name: &str, host_name: &str) -> Resource {
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
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                format!("{}:{}", host_name, std::process::id()),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::PROCESS_PID,
                i64::from(std::process::id()),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::PROCESS_RUNTIME_NAME,
                "rust",
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::PROCESS_EXECUTABLE_NAME,
                std::env::current_exe()
                    .ok()
                    .and_then(|p| {
                        p.file_name()
                            .map(|name| name.to_string_lossy().into_owned())
                    })
                    .unwrap_or_default(),
            ),
        ])
        .build()
}

pub(crate) fn init_logger_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    resource: Resource,
) -> Result<SdkLoggerProvider, crate::Error> {
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
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build())
}

pub(crate) fn init_tracer_provider(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    resource: Resource,
) -> Result<SdkTracerProvider, crate::Error> {
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
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build())
}

pub(crate) fn init_process_metrics(meter_provider: &SdkMeterProvider) {
    let meter = meter_provider.meter("process");
    let start_time = Instant::now();

    // process.uptime
    let _uptime = meter
        .f64_observable_gauge("process.uptime")
        .with_description("The time the process has been running.")
        .with_unit("s")
        .with_callback(move |observer: &dyn AsyncInstrument<f64>| {
            observer.observe(start_time.elapsed().as_secs_f64(), &[]);
        })
        .build();

    // process.memory.usage (RSS) - Linux only
    #[cfg(target_os = "linux")]
    {
        let _rss = meter
            .u64_observable_gauge("process.memory.usage")
            .with_description("The amount of physical memory in use.")
            .with_unit("By")
            .with_callback(|observer: &dyn AsyncInstrument<u64>| {
                if let Some(rss) = read_proc_status_field("VmRSS") {
                    observer.observe(rss * 1024, &[]);
                }
            })
            .build();

        let _vsz = meter
            .u64_observable_gauge("process.memory.virtual")
            .with_description("The amount of virtual memory in use.")
            .with_unit("By")
            .with_callback(|observer: &dyn AsyncInstrument<u64>| {
                if let Some(vsz) = read_proc_status_field("VmSize") {
                    observer.observe(vsz * 1024, &[]);
                }
            })
            .build();

        let _threads = meter
            .u64_observable_gauge("process.thread.count")
            .with_description("Process threads count.")
            .with_callback(|observer: &dyn AsyncInstrument<u64>| {
                if let Some(threads) = read_proc_status_field("Threads") {
                    observer.observe(threads, &[]);
                }
            })
            .build();
    }
}

#[cfg(target_os = "linux")]
fn read_proc_status_field(field: &str) -> Option<u64> {
    let contents = std::fs::read_to_string("/proc/self/status").ok()?;
    for line in contents.lines() {
        if line.starts_with(field) {
            return line.split_whitespace().nth(1)?.parse().ok();
        }
    }
    None
}

pub(crate) fn init_metrics(
    new_relic_otlp_endpoint: &str,
    new_relic_license_key: &str,
    resource: Resource,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, crate::Error> {
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
        .with_resource(resource)
        .build())
}

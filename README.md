# Easy Init NewRelic OpenTelemetry

This crate provides a subscriber for OpenTelemetry that sends spans and metrics to New Relic.

## Example
```rust
use easy_init_newrelic_opentelemetry::NewRelicSubscriberInitializer;
use time::macros::offset;

fn main() {
    NewRelicSubscriberInitializer::default()
        .newrelic_otlp_endpoint("http://localhost:4317")
        .newrelic_license_key("1234567890abcdef1234567890abcdef12345678")
        .newrelic_service_name("test-service")
        .host_name("test-host")
        .timestamps_offset(offset!(+00:00:00));
}
```

//! Canonical schema generator for `wyrd-client` public types.
//!
//! Writes `crates/wyrd-client/schemas/<name>.json` per type — the single
//! golden set: diffed by `codegen:check` and read by the schema-drift test.
//! Invoked by `mise codegen:regen`.

use std::path::Path;

fn write<T: schemars::JsonSchema>(out: &Path, name: &str) -> std::io::Result<()> {
    let mut schema = schemars::schema_for!(T);
    schema.meta_schema = Some("https://json-schema.org/draft/2020-12/schema".to_string());
    let json = serde_json::to_string_pretty(&schema)
        .expect("schema serialization should be infallible for generated contracts");
    let body = format!("{json}\n");
    std::fs::write(out.join(format!("{name}.json")), &body)?;
    Ok(())
}

fn main() -> std::io::Result<()> {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = manifest.join("schemas");
    std::fs::create_dir_all(&out)?;

    use wyrd_client::transport::{GrpcConfig, HttpConfig, MockConfig, TransportConfig};

    write::<TransportConfig>(&out, "transport_config_enum")?;
    write::<GrpcConfig>(&out, "transport_config_grpc")?;
    write::<HttpConfig>(&out, "transport_config_http")?;
    write::<MockConfig>(&out, "transport_config_mock")?;
    Ok(())
}

use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let wrangler_toml_path = Path::new(&manifest_dir).join("wrangler.toml");

    println!("cargo:rerun-if-changed=wrangler.toml");

    let wrangler_content = fs::read_to_string(&wrangler_toml_path)
        .expect("Failed to read wrangler.toml");

    let config: toml::Value = wrangler_content
        .parse()
        .expect("Failed to parse wrangler.toml");

    let vars = config
        .get("vars")
        .and_then(|v| v.as_table())
        .expect("Missing [vars] section in wrangler.toml");

    let anon_hourly = vars
        .get("ANON_HOURLY_LIMIT")
        .and_then(|v| v.as_str())
        .expect("Missing ANON_HOURLY_LIMIT in wrangler.toml");

    let anon_daily = vars
        .get("ANON_DAILY_LIMIT")
        .and_then(|v| v.as_str())
        .expect("Missing ANON_DAILY_LIMIT in wrangler.toml");

    let max_file_size = vars
        .get("MAX_FILE_SIZE_MB")
        .and_then(|v| v.as_str())
        .expect("Missing MAX_FILE_SIZE_MB in wrangler.toml");

    let retention_days = vars
        .get("DEFAULT_RETENTION_DAYS")
        .and_then(|v| v.as_str())
        .expect("Missing DEFAULT_RETENTION_DAYS in wrangler.toml");

    println!("cargo:rustc-env=ANON_HOURLY_LIMIT={}", anon_hourly);
    println!("cargo:rustc-env=ANON_DAILY_LIMIT={}", anon_daily);
    println!("cargo:rustc-env=MAX_FILE_SIZE_MB={}", max_file_size);
    println!("cargo:rustc-env=DEFAULT_RETENTION_DAYS={}", retention_days);
}

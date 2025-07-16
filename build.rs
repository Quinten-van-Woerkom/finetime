//! Build file that is used to generate artifacts that may change over time, like the leap second
//! table.

use std::fs;
use std::path::PathBuf;

fn main() {
    // Step 1: Download from the current IANA URL
    const URL: &str = "https://data.iana.org/time-zones/data/leap-seconds.list";
    let response = ureq::get(URL)
        .call()
        .expect("Failed to download leap-seconds.list from IANA");

    let text = response
        .into_string()
        .expect("Failed to read leap-seconds.list response");

    // Step 2: Parse and convert to Rust source
    let mut output = String::new();
    output.push_str("// Automatically-generated leap table based on the `leap-seconds.list` published by IANA at\n");
    output.push_str(format!("// {URL}\n").as_str());
    output.push_str("// Formatted as (Unix timestamp of UTC second, total TAI-UTC seconds)\n\n");
    output.push_str("use lazy_static::lazy_static;\n");
    output.push_str("use crate::duration::Seconds;\n\n");

    // output
    //     .push_str("/// Returns a slice that consists of the leap second table as published by \n");
    // output.push_str("/// the IERS. The backing table returned by this function is generated at\n");
    // output.push_str("/// build time in `build.rs` directly based on the data published at\n");
    // output.push_str(format!("/// {URL}\n").as_str());
    // output.push_str(
    //     "pub const fn leap_second_table_iers() -> &'static LeapSecondsTable {\n\tLEAP_SECONDS\n}\n\n",
    // );

    output.push_str("lazy_static! {\n");
    output.push_str("static ref LEAP_SECONDS: LeapSecondsTable = {\n");
    output.push_str("    let mut leap_seconds = LeapSecondsTable::default();\n");

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let parts: Vec<&str> = trimmed.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }
        let ntp_secs: i64 = parts[0].parse().expect("Invalid NTP seconds");
        let offset: i64 = parts[1].parse().expect("Invalid leap-second count");
        // We store the offset from Unix time rather than from TAI, since that makes for a more
        // direct translation. With TAI, we would also have to include an epoch shift (since our
        // `TimePoint` implementation uses an epoch of 1958-01-01 for TAI and 1970-01-01 for UTC).
        let offset = offset - 10i64;
        let unix_secs = ntp_secs - 2_208_988_800; // Convert NTP (1900) → Unix (1970)
        output.push_str(&format!(
            "    leap_seconds.insert(Seconds::new({unix_secs}), Seconds::new({offset}));\n"
        ));
    }
    output.push_str("    leap_seconds\n");
    output.push_str("};\n");
    output.push_str("}\n");

    // Step 4: Write to build output directory
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("leap_seconds.rs");
    fs::write(&out, output).expect("Failed to write leap_seconds.rs");

    println!("cargo:rerun-if-changed=build.rs");
}

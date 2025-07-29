//! Build file that is used to generate artifacts that may change over time, like the leap second
//! table.

use std::fs;
use std::path::PathBuf;

fn main() {
    let mut output = String::new();

    if std::env::var("DOCS_RS").is_ok() {
        /// Placeholder build function that is called when running on `docsrs`. Does not require internet
        /// to build, but might contain an out-of-date leap seconds table.
        const PLACEHOLDER_CONTENTS: &str = r"
// Automatically-generated leap table based on the `leap-seconds.list` published by IANA at
// https://data.iana.org/time-zones/data/leap-seconds.list
// Formatted as (Unix timestamp of UTC second, total TAI-UTC seconds)
// 
// WARNING: This version was generated without internet, so might be out-of-date.

use lazy_static::lazy_static;
use crate::duration::Seconds;

lazy_static! {
pub(crate) static ref LEAP_SECONDS: LeapSecondsTable = {
    let mut leap_seconds = LeapSecondsTable::default();
    leap_seconds.insert(Seconds::new(63072000), Seconds::new(0));
    leap_seconds.insert(Seconds::new(78796800), Seconds::new(1));
    leap_seconds.insert(Seconds::new(94694400), Seconds::new(2));
    leap_seconds.insert(Seconds::new(126230400), Seconds::new(3));
    leap_seconds.insert(Seconds::new(157766400), Seconds::new(4));
    leap_seconds.insert(Seconds::new(189302400), Seconds::new(5));
    leap_seconds.insert(Seconds::new(220924800), Seconds::new(6));
    leap_seconds.insert(Seconds::new(252460800), Seconds::new(7));
    leap_seconds.insert(Seconds::new(283996800), Seconds::new(8));
    leap_seconds.insert(Seconds::new(315532800), Seconds::new(9));
    leap_seconds.insert(Seconds::new(362793600), Seconds::new(10));
    leap_seconds.insert(Seconds::new(394329600), Seconds::new(11));
    leap_seconds.insert(Seconds::new(425865600), Seconds::new(12));
    leap_seconds.insert(Seconds::new(489024000), Seconds::new(13));
    leap_seconds.insert(Seconds::new(567993600), Seconds::new(14));
    leap_seconds.insert(Seconds::new(631152000), Seconds::new(15));
    leap_seconds.insert(Seconds::new(662688000), Seconds::new(16));
    leap_seconds.insert(Seconds::new(709948800), Seconds::new(17));
    leap_seconds.insert(Seconds::new(741484800), Seconds::new(18));
    leap_seconds.insert(Seconds::new(773020800), Seconds::new(19));
    leap_seconds.insert(Seconds::new(820454400), Seconds::new(20));
    leap_seconds.insert(Seconds::new(867715200), Seconds::new(21));
    leap_seconds.insert(Seconds::new(915148800), Seconds::new(22));
    leap_seconds.insert(Seconds::new(1136073600), Seconds::new(23));
    leap_seconds.insert(Seconds::new(1230768000), Seconds::new(24));
    leap_seconds.insert(Seconds::new(1341100800), Seconds::new(25));
    leap_seconds.insert(Seconds::new(1435708800), Seconds::new(26));
    leap_seconds.insert(Seconds::new(1483228800), Seconds::new(27));
    leap_seconds
};
}
";
        output.push_str(PLACEHOLDER_CONTENTS);
    } else {
        // Step 1: Download from the current IANA URL
        const URL: &str = "https://data.iana.org/time-zones/data/leap-seconds.list";
        let response = ureq::get(URL)
            .call()
            .expect("Failed to download leap-seconds.list from IANA");

        let text = response
            .into_string()
            .expect("Failed to read leap-seconds.list response");

        // Step 2: Parse and convert to Rust source
        output.push_str("// Automatically-generated leap table based on the `leap-seconds.list` published by IANA at\n");
        output.push_str(format!("// {URL}\n").as_str());
        output
            .push_str("// Formatted as (Unix timestamp of UTC second, total TAI-UTC seconds)\n\n");
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
        output.push_str("pub(crate) static ref LEAP_SECONDS: LeapSecondsTable = {\n");
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
    }

    // Final step: Write to build output directory
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("leap_seconds.rs");
    fs::write(&out, output).expect("Failed to write leap_seconds.rs");

    println!("cargo:rerun-if-changed=build.rs");
}

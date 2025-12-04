use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Generate build timestamp for cache busting
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", timestamp);
}

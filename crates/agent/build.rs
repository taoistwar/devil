use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest_path = Path::new(&out_dir).join("bundled_skills.tar.gz");
    
    // Create a minimal valid gzip file (empty tar.gz)
    // This is a minimal gzip file with no content
    let minimal_gzip: &[u8] = &[
        0x1f, 0x8b, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff, 0xff,
        0x00, 0x00, 0x00, 0x01, 0xae, 0x7f, 0x0d, 0x0a,
    ];
    
    fs::write(&dest_path, minimal_gzip).expect("Failed to write bundled_skills.tar.gz");
    
    // Tell Cargo to rerun this build script if the source file changes
    println!("cargo:rerun-if-changed=build.rs");
}

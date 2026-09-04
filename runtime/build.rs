fn main() {
    // Locate libgc via pkg-config when available (Homebrew on macOS, system
    // packages on Linux); fall back to the default linker search path.
    if pkg_config::probe_library("bdw-gc").is_err() {
        println!("cargo:rustc-link-lib=gc");
    }
    println!("cargo:rerun-if-changed=build.rs");
}

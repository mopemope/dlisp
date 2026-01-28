fn main() {
    println!("cargo:rustc-link-lib=gc");
    println!("cargo:rerun-if-changed=build.rs");
}

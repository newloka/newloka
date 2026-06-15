fn main() {
    // Tell Cargo to re-run this build script when the web assets change.
    println!("cargo:rerun-if-changed=../newloka_web");
}

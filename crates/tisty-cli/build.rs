fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    static_vcruntime::metabuild();
}

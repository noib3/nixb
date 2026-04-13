fn main() {
    println!("cargo::rerun-if-changed=build.rs");

    #[cfg(target_os = "macos")]
    {
        println!("cargo::rustc-link-arg=-undefined");
        println!("cargo::rustc-link-arg=dynamic_lookup");
    }
}

#![expect(missing_docs)]

fn main() {
    if let Ok(version) = rustc_version::version_meta()
        && version.channel == rustc_version::Channel::Nightly
    {
        println!("cargo:rustc-cfg=nightly");
    }
}

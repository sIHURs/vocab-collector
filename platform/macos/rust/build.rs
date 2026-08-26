fn main() {
    #[cfg(target_os = "macos")]
    {
        use std::path::PathBuf;

        let package_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap())
            .parent()
            .unwrap()
            .to_owned();
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
        println!("cargo::metadata=final-link-arg=-Wl,-rpath,/usr/lib/swift");
        println!(
            "cargo:rerun-if-changed={}",
            package_dir.join("Package.swift").display()
        );
        println!(
            "cargo:rerun-if-changed={}",
            package_dir.join("Sources").display()
        );
        swift_rs::SwiftLinker::new("15.0")
            .with_package("VocabMacBridge", package_dir)
            .link();
    }
}

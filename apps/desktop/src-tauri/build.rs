fn main() {
    #[cfg(target_os = "macos")]
    {
        // Rust test binaries do not inherit the standard Swift runtime rpath.
        // macOS 15 ships these libraries in the shared cache at this location.
        println!("cargo:rustc-link-arg=-Wl,-rpath,/usr/lib/swift");
        println!("cargo:rerun-if-changed=../../../platform/macos/Package.swift");
        println!("cargo:rerun-if-changed=../../../platform/macos/Sources");
        swift_rs::SwiftLinker::new("15.0")
            .with_package("VocabMacBridge", "../../../platform/macos")
            .link();
    }
    tauri_build::build();
}

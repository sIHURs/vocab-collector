fn main() {
    #[cfg(target_os = "macos")]
    {
        println!("cargo:rerun-if-changed=../../../platform/macos/Package.swift");
        println!("cargo:rerun-if-changed=../../../platform/macos/Sources");
        swift_rs::SwiftLinker::new("15.0")
            .with_package("VocabMacBridge", "../../../platform/macos")
            .link();
    }
    tauri_build::build();
}

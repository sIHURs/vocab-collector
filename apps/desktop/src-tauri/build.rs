fn forward_dependency_link_args() {
    for (key, value) in std::env::vars() {
        if key.starts_with("DEP_") && key.ends_with("_FINAL_LINK_ARG") {
            println!("cargo:rustc-link-arg={value}");
        }
    }
}

fn main() {
    forward_dependency_link_args();
    tauri_build::build();
}

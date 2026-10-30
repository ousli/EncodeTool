fn main() {
    // rust-embed needs the folder to exist even before the frontend is built.
    let _ = std::fs::create_dir_all("web/dist");
    println!("cargo:rerun-if-changed=web/dist");
}

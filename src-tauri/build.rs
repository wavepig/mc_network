#[path = "build_support/easytier_pack.rs"]
mod easytier_pack;

fn main() {
    easytier_pack::prepare();
    tauri_build::build();
}

fn main() {
    let output_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "renders/shutdown_frames".to_string());
    rotating_terminal::run_export_shutdown(output_dir);
}

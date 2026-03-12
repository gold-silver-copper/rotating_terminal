fn main() {
    let output_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "renders/rotation_frames".to_string());
    rotating_terminal::run_export_rotation(output_dir);
}

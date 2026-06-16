fn main() {
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
        tauri_build::AppManifest::new().commands(&[
            "get_setup_status",
            "list_agents",
            "get_dashboard_status",
            "get_image_status",
            "plan_run",
            "launch_run",
        ]),
    ))
    .expect("failed to build Tauri application manifest");
}

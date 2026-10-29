fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new().app_manifest(
            tauri_build::AppManifest::new().commands(&[
                "get_config",
                "save_config",
                "close_settings",
                "heartbeat",
            ]),
        ),
    )
    .expect("falha ao gerar o manifesto de permissões do Tauri");
}

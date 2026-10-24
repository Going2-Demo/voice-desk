//! Núcleo do Voice Desk: montagem do app, plugins e janelas.

pub mod comandos;
pub mod configuracao;
pub mod ditado;
pub mod nucleo;
pub mod plataforma;

use tauri::Manager;
use tauri_specta::{Builder, collect_commands};

pub fn construtor_ipc() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![comandos::app_sobre])
}

/// Gera `src/ipc/bindings.ts` (tipos e funções de todos os comandos).
pub fn exportar_bindings(destino: &std::path::Path) -> Result<(), String> {
    construtor_ipc()
        .export(specta_typescript::Typescript::default(), destino)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let ipc = construtor_ipc();
    #[cfg(debug_assertions)]
    if let Err(e) = exportar_bindings(
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ipc/bindings.ts"),
    ) {
        eprintln!("não foi possível gerar bindings.ts: {e}");
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(janela) = app.get_webview_window("principal") {
                let _ = janela.show();
                let _ = janela.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(ipc.invoke_handler())
        .setup(move |app| {
            ipc.mount_events(app);
            let pasta = app.path().app_data_dir()?;
            let caminhos = nucleo::caminhos::Caminhos::resolver(&pasta);
            std::fs::create_dir_all(caminhos.logs())?;
            let guarda = nucleo::log::iniciar(&caminhos.logs())?;
            app.manage(guarda);
            tracing::info!(evento = "app_iniciado");
            let conexao = nucleo::banco::abrir(&caminhos.banco())?;
            app.manage(std::sync::Mutex::new(conexao));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("erro ao iniciar o Voice Desk");
}

#[cfg(test)]
mod testes {
    #[test]
    fn bindings_versionados_estao_atualizados() {
        let atual = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../src/ipc/bindings.ts");
        if std::env::var_os("VOICE_DESK_REGERAR_BINDINGS").is_some() {
            super::exportar_bindings(&atual).unwrap();
        }
        let tmp = tempfile::tempdir().unwrap();
        let novo = tmp.path().join("bindings.ts");
        super::exportar_bindings(&novo).unwrap();
        assert_eq!(
            std::fs::read_to_string(&atual).unwrap_or_default(),
            std::fs::read_to_string(&novo).unwrap(),
            "src/ipc/bindings.ts desatualizado: rode `cargo test` com VOICE_DESK_REGERAR_BINDINGS=1 ou o app em dev"
        );
    }
}

//! Núcleo do Voice Desk: montagem do app, plugins e janelas.

pub mod bandeja;
pub mod chaves;
pub mod comandos;
pub mod configuracao;
pub mod ditado;
pub mod nucleo;
pub mod plataforma;

use tauri::Manager;
use tauri_specta::{Builder, collect_commands};

pub fn construtor_ipc() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new().commands(collect_commands![
        comandos::app_sobre,
        bandeja::bandeja_estado,
        bandeja::bandeja_iniciar_com_o_sistema,
        bandeja::bandeja_definir_iniciar_com_o_sistema,
        comandos::chaves_estado,
        comandos::chaves_salvar,
        comandos::chaves_testar,
        comandos::chaves_apagar,
        comandos::chaves_abrir_pagina,
    ])
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
            bandeja::abrir_principal(app, None);
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec![bandeja::ARGUMENTO_OCULTO]),
        ))
        .plugin(tauri_plugin_dialog::init())
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
            bandeja::iniciar(app.handle())?;
            // Aberto pelo login do sistema: só o ícone, sem janela (RF-05.2).
            if !std::env::args().any(|a| a == bandeja::ARGUMENTO_OCULTO) {
                bandeja::abrir_principal(app.handle(), None);
            }
            let cofre: comandos::CofreEstado =
                std::sync::Arc::new(chaves::cofre::CofreSistema::novo());
            app.manage(cofre);
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("erro ao iniciar o Voice Desk")
        .run(|_, evento| {
            // O app vive na bandeja: fechar a última janela não encerra; só `exit` explícito (Sair).
            if let tauri::RunEvent::ExitRequested {
                api, code: None, ..
            } = evento
            {
                api.prevent_exit();
            }
        });
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

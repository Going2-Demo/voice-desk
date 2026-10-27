//! Núcleo do Voice Desk: montagem do app, plugins e janelas.

pub mod atalhos;
pub mod audio;
pub mod bandeja;
pub mod chaves;
pub mod comandos;
pub mod configuracao;
pub mod ditado;
pub mod nucleo;
pub mod permissoes;
pub mod plataforma;
pub mod primeiro_uso;
pub mod transcricao;
pub mod vocabulario;

use tauri::Manager;
use tauri_specta::{Builder, collect_commands, collect_events};

pub fn construtor_ipc() -> Builder<tauri::Wry> {
    Builder::<tauri::Wry>::new()
        .commands(collect_commands![
            comandos::app_sobre,
            comandos::app_abrir_projeto,
            comandos::atalhos_obter,
            comandos::atalhos_validar,
            comandos::atalhos_salvar,
            comandos::ditado_alternar,
            comandos::ditado_abrir_chaves,
            comandos::transcricao_obter,
            comandos::transcricao_salvar,
            bandeja::bandeja_estado,
            bandeja::bandeja_iniciar_com_o_sistema,
            bandeja::bandeja_definir_iniciar_com_o_sistema,
            comandos::chaves_estado,
            comandos::chaves_salvar,
            comandos::chaves_testar,
            comandos::chaves_apagar,
            comandos::chaves_abrir_pagina,
            permissoes::permissoes_estado,
            permissoes::permissoes_pedir,
            permissoes::permissoes_abrir_configuracoes,
            primeiro_uso::primeiro_uso_estado,
            primeiro_uso::primeiro_uso_pular,
            primeiro_uso::primeiro_uso_cumprir,
            primeiro_uso::primeiro_uso_concluir,
            comandos::vocabulario_listar,
            comandos::vocabulario_adicionar,
            comandos::vocabulario_editar,
            comandos::vocabulario_remover,
        ])
        .events(collect_events![
            ditado::efeitos_tauri::DitadoEstado,
            ditado::efeitos_tauri::DitadoNivel,
        ])
}

/// Janela do gravador: pílula transparente, sempre no topo, que não toma o foco (RF-01.2).
fn criar_janela_gravador(app: &tauri::App) -> tauri::Result<()> {
    tauri::WebviewWindowBuilder::new(
        app,
        ditado::efeitos_tauri::JANELA_GRAVADOR,
        tauri::WebviewUrl::App("gravador.html".into()),
    )
    .title("Voice Desk — Gravador")
    .inner_size(460.0, 64.0)
    .decorations(false)
    .transparent(true)
    .shadow(false)
    .resizable(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .visible_on_all_workspaces(true)
    .focusable(false)
    .focused(false)
    .visible(false)
    .build()?;
    Ok(())
}

/// Liga o ditado: efeitos reais, relógio do gravador e atalho global salvo (ou o padrão).
fn montar_ditado(app: &tauri::App) -> Result<(), Box<dyn std::error::Error>> {
    let efeitos = ditado::efeitos_tauri::EfeitosTauri::novo(app.handle().clone());
    let ditado = ditado::servico::Ditado::novo(std::sync::Arc::clone(&efeitos));
    let para_esc = std::sync::Arc::clone(&ditado);
    let _ = efeitos
        .ao_esc
        .set(std::sync::Arc::new(move || para_esc.esc()));
    let para_tique = std::sync::Arc::clone(&ditado);
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(200));
            para_tique.tique();
        }
    });
    let registrado = atalhos::registro::Registrado::default();
    let combinacao = {
        let banco = app.state::<std::sync::Mutex<rusqlite::Connection>>();
        let conexao = banco.lock().map_err(|_| "trava do banco")?;
        atalhos::servico::obter(&conexao, plataforma::atual())?
    };
    if let Err(e) = atalhos::registro::registrar(
        app.handle(),
        &registrado,
        &combinacao,
        &comandos::acionador(&ditado),
    ) {
        tracing::warn!(evento = "atalho_nao_registrado", codigo = %e.codigo);
    }
    app.manage(registrado);
    app.manage(ditado);
    Ok(())
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
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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
            let pasta_chaves = caminhos.raiz().to_path_buf();
            app.manage(caminhos);
            app.manage(std::sync::Mutex::new(conexao));
            bandeja::iniciar(app.handle())?;
            // Aberto pelo login do sistema: só o ícone, sem janela (RF-05.2).
            if !std::env::args().any(|a| a == bandeja::ARGUMENTO_OCULTO) {
                bandeja::abrir_principal(app.handle(), None);
            }
            let cofre: comandos::CofreEstado =
                std::sync::Arc::new(chaves::cofre::CofreArquivo::novo(&pasta_chaves)?);
            app.manage(cofre);
            criar_janela_gravador(app)?;
            montar_ditado(app)?;
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

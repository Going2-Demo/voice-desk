//! Lista única de comandos e eventos, um bloco por área, na ordem da seção 10 da arquitetura.
//! Cada história só acrescenta linhas no bloco da própria área.

use crate::atalhos::{registro, servico as atalhos_servico, validacao};
use crate::chaves::cofre::Cofre;
use crate::chaves::{EstadoChave, Provedor, groq, servico as chaves};
use crate::ditado::efeitos_tauri::EfeitosTauri;
use crate::ditado::servico::Ditado;
use crate::nucleo::{ErroApp, relogio::Relogio};
use crate::plataforma;
use crate::transcricao::{self, ConfigTranscricao};
use rusqlite::Connection;
use serde::Serialize;
use specta::Type;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, State, Wry};
use tauri_plugin_opener::OpenerExt;

/// Cofre de chaves registrado no estado do app (o do sistema; nos testes, um em memória).
pub type CofreEstado = Arc<dyn Cofre>;
type Banco = Mutex<Connection>;

fn banco<'a>(b: &'a Banco) -> Result<std::sync::MutexGuard<'a, Connection>, ErroApp> {
    b.lock().map_err(|_| ErroApp::interno("banco bloqueado"))
}

pub type DitadoApp = Ditado<Arc<EfeitosTauri>>;

#[derive(Debug, Clone, Serialize, Type)]
pub struct Sobre {
    pub versao: String,
    pub commit: String,
    /// Identificador SPDX da licença do pacote (RF-27.4).
    pub licenca: String,
    /// Página pública do projeto (RF-27.4).
    pub url_projeto: String,
}

/// Página pública do projeto (F-27), publicada pelo GitHub Pages.
pub const URL_PROJETO: &str = "https://going2-demo.github.io/voice-desk/";

// --- fundacao / app -----------------------------------------------------------------------
#[tauri::command]
#[specta::specta]
pub fn app_sobre() -> Result<Sobre, ErroApp> {
    Ok(Sobre {
        versao: env!("CARGO_PKG_VERSION").into(),
        commit: option_env!("VOICE_DESK_COMMIT").unwrap_or("local").into(),
        licenca: env!("CARGO_PKG_LICENSE").into(),
        url_projeto: URL_PROJETO.into(),
    })
}

/// Abre a página do projeto no navegador padrão (RF-27.4).
#[tauri::command]
#[specta::specta]
pub fn app_abrir_projeto(app: tauri::AppHandle) -> Result<(), ErroApp> {
    app.opener()
        .open_url(URL_PROJETO, None::<&str>)
        .map_err(ErroApp::interno)
}

// --- ditado, audio, atalhos, plataforma-*, permissoes -------------------------------------
#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct Atalho {
    /// Forma canônica, ex.: `Ctrl+Alt+Space`.
    pub combinacao: String,
    /// Texto para mostrar: `⌃⌥Espaço` no Mac, `Ctrl+Alt+Espaço` no Windows.
    pub exibicao: String,
    /// Sugestão do primeiro uso nesta plataforma.
    pub padrao: String,
}

fn atalho_de(combinacao: String) -> Atalho {
    let p = plataforma::atual();
    Atalho {
        exibicao: validacao::exibir(&combinacao, p),
        padrao: atalhos_servico::padrao(p).to_string(),
        combinacao,
    }
}

fn banco_travado(
    banco: &Mutex<Connection>,
) -> Result<std::sync::MutexGuard<'_, Connection>, ErroApp> {
    banco.lock().map_err(|_| ErroApp::interno("trava do banco"))
}

#[tauri::command]
#[specta::specta]
pub fn atalhos_obter(banco: State<'_, Mutex<Connection>>) -> Result<Atalho, ErroApp> {
    let c = banco_travado(&banco)?;
    Ok(atalho_de(atalhos_servico::obter(&c, plataforma::atual())?))
}

/// Confere o formato e conflitos sem gravar (resposta rápida enquanto o usuário digita).
#[tauri::command]
#[specta::specta]
pub fn atalhos_validar(
    combinacao: String,
    banco: State<'_, Mutex<Connection>>,
) -> Result<Atalho, ErroApp> {
    let c = banco_travado(&banco)?;
    Ok(atalho_de(atalhos_servico::validar_para_salvar(
        &c,
        plataforma::atual(),
        &combinacao,
    )?))
}

/// Valida, registra no sistema (vale sem reiniciar) e só então grava.
#[tauri::command]
#[specta::specta]
pub fn atalhos_salvar(
    combinacao: String,
    app: AppHandle<Wry>,
    banco: State<'_, Mutex<Connection>>,
    registrado: State<'_, registro::Registrado>,
    ditado: State<'_, Arc<DitadoApp>>,
) -> Result<Atalho, ErroApp> {
    let c = banco_travado(&banco)?;
    let canonica = atalhos_servico::validar_para_salvar(&c, plataforma::atual(), &combinacao)?;
    registro::registrar(&app, &registrado, &canonica, &acionador(&ditado))?;
    atalhos_servico::gravar(&c, &canonica)?;
    Ok(atalho_de(canonica))
}

/// Mesmo efeito do atalho global (botão "Testar ditado" e testes).
#[tauri::command]
#[specta::specta]
pub fn ditado_alternar(ditado: State<'_, Arc<DitadoApp>>) {
    ditado.atalho();
}

/// Atalho do aviso "cadastre a chave" no gravador: abre a tela de chaves.
#[tauri::command]
#[specta::specta]
pub fn ditado_abrir_chaves(ditado: State<'_, Arc<DitadoApp>>) {
    ditado.abrir_chaves();
}

pub fn acionador(ditado: &Arc<DitadoApp>) -> Arc<dyn Fn() + Send + Sync> {
    let d = Arc::clone(ditado);
    Arc::new(move || d.atalho())
}

// --- chaves, transcricao, transcricao-local, refinamento ----------------------------------
#[tauri::command]
#[specta::specta]
pub fn chaves_estado(
    cofre: State<'_, CofreEstado>,
    bd: State<'_, Banco>,
) -> Result<Vec<EstadoChave>, ErroApp> {
    let conexao = banco(&bd)?;
    Provedor::todos()
        .iter()
        .map(|p| chaves::estado(cofre.as_ref(), &conexao, *p))
        .collect()
}

/// Valida o formato, testa a chave no provedor (até 10 s) e guarda no cofre.
#[tauri::command]
#[specta::specta]
pub async fn chaves_salvar(
    cofre: State<'_, CofreEstado>,
    bd: State<'_, Banco>,
    provedor: Provedor,
    chave: String,
) -> Result<EstadoChave, ErroApp> {
    let chave = chaves::validar_formato(&chave)?;
    let teste = testar_no_provedor(provedor, &chave).await;
    let conexao = banco(&bd)?;
    chaves::registrar(cofre.as_ref(), &conexao, provedor, &chave, &teste)
}

/// Testa de novo a chave já guardada (por exemplo, depois de uma falha de rede).
#[tauri::command]
#[specta::specta]
pub async fn chaves_testar(
    cofre: State<'_, CofreEstado>,
    bd: State<'_, Banco>,
    provedor: Provedor,
) -> Result<EstadoChave, ErroApp> {
    let chave = chaves::obter(cofre.as_ref(), provedor)?;
    let teste = testar_no_provedor(provedor, &chave).await;
    let conexao = banco(&bd)?;
    chaves::registrar_teste(cofre.as_ref(), &conexao, provedor, &teste)
}

#[tauri::command]
#[specta::specta]
pub fn chaves_apagar(
    cofre: State<'_, CofreEstado>,
    bd: State<'_, Banco>,
    provedor: Provedor,
) -> Result<EstadoChave, ErroApp> {
    let conexao = banco(&bd)?;
    chaves::apagar(cofre.as_ref(), &conexao, provedor)
}

/// Abre no navegador padrão a página do provedor onde se obtém a chave (RF-04.4).
#[tauri::command]
#[specta::specta]
pub fn chaves_abrir_pagina(app: tauri::AppHandle, provedor: Provedor) -> Result<(), ErroApp> {
    app.opener()
        .open_url(provedor.pagina_da_chave(), None::<&str>)
        .map_err(ErroApp::interno)
}

async fn testar_no_provedor(provedor: Provedor, chave: &str) -> crate::chaves::ResultadoTeste {
    match provedor {
        Provedor::Groq => groq::testar(groq::URL_BASE, chave).await,
    }
}

/// Provedor, modelo e idioma da transcrição de um modo (sem `modo_id`, o do modo ativo).
#[tauri::command]
#[specta::specta]
pub fn transcricao_obter(
    modo_id: Option<String>,
    bd: State<'_, Banco>,
) -> Result<ConfigTranscricao, ErroApp> {
    let c = banco(&bd)?;
    let modo = match modo_id {
        Some(id) => Some(id),
        None => transcricao::config::modo_ativo(&c)?,
    };
    transcricao::config::obter(&c, modo.as_deref())
}

/// Grava provedor, modelo e idioma da transcrição do modo (RF-02.1 e RF-02.2).
#[tauri::command]
#[specta::specta]
pub fn transcricao_salvar(
    modo_id: String,
    config: ConfigTranscricao,
    bd: State<'_, Banco>,
    relogio: State<'_, Relogio>,
) -> Result<ConfigTranscricao, ErroApp> {
    let c = banco(&bd)?;
    let maquina = crate::bandeja::maquina_id(&c)?;
    transcricao::config::salvar(&c, &relogio, &maquina, &modo_id, &config)
}

// --- bandeja, primeiro-uso ----------------------------------------------------------------
// --- modos (+ programas, atalho, gatilho), contexto ---------------------------------------
// --- vocabulario, substituicoes, limpeza --------------------------------------------------
// --- historico, retencao, retranscrever, exportar-csv -------------------------------------
// --- configuracao-arquivo, sincronizacao --------------------------------------------------

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn sobre_traz_a_versao_do_pacote() {
        let s = app_sobre().unwrap();
        assert_eq!(s.versao, env!("CARGO_PKG_VERSION"));
        assert!(!s.commit.is_empty());
    }

    #[test]
    fn sobre_traz_licenca_e_pagina_do_projeto() {
        let s = app_sobre().unwrap();
        assert_eq!(s.licenca, "MIT");
        assert!(s.url_projeto.starts_with("https://"));
        assert_eq!(s.url_projeto, URL_PROJETO);
    }
}

//! Comandos `permissoes_*` (F-06, F-03, F-13): estado das permissões do sistema e atalho para a
//! tela do sistema onde o usuário as concede. A detecção em si vive em `plataforma/permissoes`.

use crate::nucleo::ErroApp;
use crate::plataforma::{
    self,
    permissoes::{self as sistema, Permissao, Situacao},
};
use serde::{Deserialize, Serialize};
use specta::Type;
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum PermissaoApp {
    Microfone,
    Acessibilidade,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum SituacaoApp {
    Concedida,
    Negada,
    Pendente,
    NaoSeAplica,
}

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EstadoPermissoes {
    /// `macos` ou `windows`: a tela escolhe os textos (Acessibilidade só no Mac).
    pub plataforma: String,
    pub microfone: SituacaoApp,
    pub acessibilidade: SituacaoApp,
}

impl From<PermissaoApp> for Permissao {
    fn from(p: PermissaoApp) -> Self {
        match p {
            PermissaoApp::Microfone => Permissao::Microfone,
            PermissaoApp::Acessibilidade => Permissao::Acessibilidade,
        }
    }
}

impl From<Situacao> for SituacaoApp {
    fn from(s: Situacao) -> Self {
        match s {
            Situacao::Concedida => SituacaoApp::Concedida,
            Situacao::Negada => SituacaoApp::Negada,
            Situacao::Pendente => SituacaoApp::Pendente,
            Situacao::NaoSeAplica => SituacaoApp::NaoSeAplica,
        }
    }
}

fn estado_atual() -> EstadoPermissoes {
    EstadoPermissoes {
        plataforma: plataforma::atual().to_string(),
        microfone: sistema::situacao(Permissao::Microfone).into(),
        acessibilidade: sistema::situacao(Permissao::Acessibilidade).into(),
    }
}

#[tauri::command]
#[specta::specta]
pub fn permissoes_estado() -> Result<EstadoPermissoes, ErroApp> {
    Ok(estado_atual())
}

/// Provoca o pedido do sistema (quando existe) e devolve o estado novo de todas.
#[tauri::command]
#[specta::specta]
pub async fn permissoes_pedir(permissao: PermissaoApp) -> Result<EstadoPermissoes, ErroApp> {
    // Abrir o microfone bloqueia até o usuário responder ao diálogo: fora da thread principal.
    tauri::async_runtime::spawn_blocking(move || {
        sistema::pedir(permissao.into());
        estado_atual()
    })
    .await
    .map_err(ErroApp::interno)
}

#[tauri::command]
#[specta::specta]
pub fn permissoes_abrir_configuracoes(
    app: tauri::AppHandle,
    permissao: PermissaoApp,
) -> Result<(), ErroApp> {
    let endereco = sistema::endereco_configuracoes(permissao.into()).ok_or_else(|| {
        ErroApp::novo(
            "permissao_inexistente",
            "Esta permissão não existe neste sistema.",
        )
    })?;
    app.opener()
        .open_url(endereco, None::<&str>)
        .map_err(ErroApp::interno)
}

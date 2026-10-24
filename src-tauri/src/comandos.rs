//! Lista única de comandos e eventos, um bloco por área, na ordem da seção 10 da arquitetura.
//! Cada história só acrescenta linhas no bloco da própria área.

use crate::nucleo::ErroApp;
use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, Serialize, Type)]
pub struct Sobre {
    pub versao: String,
    pub commit: String,
}

// --- fundacao / app -----------------------------------------------------------------------
#[tauri::command]
#[specta::specta]
pub fn app_sobre() -> Result<Sobre, ErroApp> {
    Ok(Sobre {
        versao: env!("CARGO_PKG_VERSION").into(),
        commit: option_env!("VOICE_DESK_COMMIT").unwrap_or("local").into(),
    })
}

// --- ditado, audio, atalhos, plataforma-*, permissoes -------------------------------------
// --- chaves, transcricao, transcricao-local, refinamento ----------------------------------
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
}

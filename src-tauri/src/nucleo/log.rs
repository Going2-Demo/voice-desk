//! Logs em arquivo diário (7 arquivos). Só eventos da lista fechada abaixo; nunca texto ditado,
//! contexto, áudio ou chave de API (RNF-09). Nada de `println!`.

use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Lista fechada de eventos permitidos. Acrescentar um evento exige passar pelo Otávio.
pub const EVENTOS: &[&str] = &[
    "app_iniciado",
    "migracao_aplicada",
    "erro_interno",
    "permissoes_lidas",
];

pub fn iniciar(pasta: &Path) -> Result<WorkerGuard, String> {
    let arquivo = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("voice-desk")
        .filename_suffix("log")
        .max_log_files(7)
        .build(pasta)
        .map_err(|e| e.to_string())?;
    let (escritor, guarda) = tracing_appender::non_blocking(arquivo);
    tracing_subscriber::fmt()
        .with_writer(escritor)
        .with_ansi(false)
        .try_init()
        .map_err(|e| e.to_string())?;
    Ok(guarda)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn eventos_conhecidos() {
        assert!(EVENTOS.contains(&"app_iniciado"));
    }

    #[test]
    fn cria_arquivo_de_log_na_pasta() {
        let pasta = tempfile::tempdir().unwrap();
        let _guarda = iniciar(pasta.path()).unwrap();
        tracing::info!(evento = "app_iniciado");
        drop(_guarda);
        let achou = std::fs::read_dir(pasta.path()).unwrap().any(|e| {
            e.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("voice-desk.")
        });
        assert!(achou);
    }
}

//! Etapa `colar`: provisória no F-00 (passa adiante); a história da área preenche.

use super::{Contexto, Fluxo};
use crate::nucleo::ErroApp;

pub fn executar(_contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    Ok(Fluxo::Seguir)
}

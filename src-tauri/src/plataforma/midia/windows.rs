//! Windows: controles de mídia do sistema (GlobalSystemMediaTransportControls). Pausa cada sessão
//! que está tocando e guarda o id do aplicativo para retomar só essas.

use super::{Controle, Pausada};
use ::windows::Media::Control::{
    GlobalSystemMediaTransportControlsSessionManager as Gerente,
    GlobalSystemMediaTransportControlsSessionPlaybackStatus as Estado,
};

pub struct Smtc;

impl Controle for Smtc {
    fn pausar_tocando(&self) -> Vec<Pausada> {
        let mut pausadas = Vec::new();
        let Ok(gerente) = Gerente::RequestAsync().and_then(|o| o.get()) else {
            return pausadas;
        };
        let Ok(sessoes) = gerente.GetSessions() else {
            return pausadas;
        };
        for sessao in sessoes {
            let tocando = sessao
                .GetPlaybackInfo()
                .and_then(|i| i.PlaybackStatus())
                .is_ok_and(|e| e == Estado::Playing);
            if !tocando {
                continue;
            }
            let aceitou = sessao
                .TryPauseAsync()
                .and_then(|o| o.get())
                .unwrap_or(false);
            if aceitou && let Ok(id) = sessao.SourceAppUserModelId() {
                pausadas.push(Pausada(id.to_string()));
            }
        }
        pausadas
    }

    fn retomar(&self, pausadas: &[Pausada]) {
        let Ok(gerente) = Gerente::RequestAsync().and_then(|o| o.get()) else {
            return;
        };
        let Ok(sessoes) = gerente.GetSessions() else {
            return;
        };
        for sessao in sessoes {
            let id = sessao.SourceAppUserModelId().map(|i| i.to_string());
            if id.is_ok_and(|id| pausadas.iter().any(|p| p.0 == id)) {
                let _ = sessao.TryPlayAsync().and_then(|o| o.get());
            }
        }
    }
}

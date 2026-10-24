//! Registro do atalho global no sistema (tauri-plugin-global-shortcut). Trocar o atalho vale na
//! hora, sem reiniciar o app (RF-01.1).

use crate::nucleo::ErroApp;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Runtime};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

/// Combinação hoje registrada, para poder desfazer na troca.
#[derive(Default)]
pub struct Registrado(pub Mutex<Option<String>>);

fn converter(canonica: &str) -> Result<Shortcut, ErroApp> {
    canonica.parse::<Shortcut>().map_err(|e| {
        ErroApp::interno(format!(
            "combinação não convertida em atalho do sistema: {e}"
        ))
    })
}

fn ligar<R: Runtime>(
    app: &AppHandle<R>,
    atalho: Shortcut,
    ao_acionar: &Arc<dyn Fn() + Send + Sync>,
) -> Result<(), tauri_plugin_global_shortcut::Error> {
    let f = Arc::clone(ao_acionar);
    app.global_shortcut()
        .on_shortcut(atalho, move |_, _, evento| {
            if evento.state() == ShortcutState::Pressed {
                f();
            }
        })
}

/// Registra `canonica` (trocando a anterior). Se o sistema recusar, a anterior continua valendo.
pub fn registrar<R: Runtime>(
    app: &AppHandle<R>,
    estado: &Registrado,
    canonica: &str,
    ao_acionar: &Arc<dyn Fn() + Send + Sync>,
) -> Result<(), ErroApp> {
    let novo = converter(canonica)?;
    let mut atual = estado
        .0
        .lock()
        .map_err(|_| ErroApp::interno("trava do atalho"))?;
    if atual.as_deref() == Some(canonica) {
        return Ok(());
    }
    let anterior = atual.clone();
    if let Some(a) = &anterior
        && let Ok(velho) = converter(a)
    {
        let _ = app.global_shortcut().unregister(velho);
    }
    let registro = ligar(app, novo, ao_acionar);
    match registro {
        Ok(()) => {
            *atual = Some(canonica.to_string());
            Ok(())
        }
        Err(e) => {
            tracing::warn!(evento = "atalho_recusado_pelo_sistema", detalhe = %e);
            // Devolve o atalho que valia, para o app não ficar sem nenhum.
            *atual = anterior
                .as_deref()
                .and_then(|a| converter(a).ok())
                .filter(|velho| ligar(app, *velho, ao_acionar).is_ok())
                .and(anterior);
            Err(ErroApp::novo(
                "atalho_em_uso_pelo_sistema",
                "Outro programa já usa essa combinação; escolha outra.",
            )
            .com_campo("combinacao"))
        }
    }
}

//! Escuta da tecla Esc sem consumi-la (RF-01.4): o programa em foco continua recebendo o Esc;
//! o Voice Desk só conta as pressões. Usa o handy-keys em modo de escuta (sem bloqueio).

use handy_keys::{Hotkey, HotkeyManager, HotkeyState, Key, Modifiers};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

pub struct EscutaEsc {
    ativa: Arc<AtomicBool>,
}

impl EscutaEsc {
    /// Começa a escutar; `ao_esc` roda a cada vez que Esc é pressionado. Falha (log, sem texto de
    /// usuário) se o sistema negar a escuta, ex.: acessibilidade não concedida no Mac.
    pub fn iniciar(ao_esc: impl Fn() + Send + 'static) -> Result<Self, String> {
        let gerente = HotkeyManager::new().map_err(|e| e.to_string())?;
        let esc = Hotkey::new(Modifiers::empty(), Key::Escape).map_err(|e| e.to_string())?;
        let id = gerente.register(esc).map_err(|e| e.to_string())?;
        let ativa = Arc::new(AtomicBool::new(true));
        let viva = Arc::clone(&ativa);
        std::thread::spawn(move || {
            while viva.load(Ordering::Relaxed) {
                while let Some(evento) = gerente.try_recv() {
                    if evento.id == id && evento.state == HotkeyState::Pressed {
                        ao_esc();
                    }
                }
                std::thread::sleep(Duration::from_millis(15));
            }
            let _ = gerente.unregister(id);
        });
        Ok(Self { ativa })
    }
}

impl Drop for EscutaEsc {
    fn drop(&mut self) {
        self.ativa.store(false, Ordering::Relaxed);
    }
}

//! Teclas simuladas no macOS por CGEvent. O Command vai como flag do próprio evento da tecla V
//! (não como tecla modificadora separada), o que o sistema aceita de forma mais confiável.

use core_graphics::event::{CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode};
use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
use std::time::Duration;

const KVK_ANSI_V: CGKeyCode = 9;
const KVK_RETURN: CGKeyCode = 36;
/// Pausa entre pressionar e soltar, para o app em foco registrar a tecla.
const PAUSA_ENTRE_EVENTOS: Duration = Duration::from_millis(15);

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

/// Evento de teclado do tipo `Press`/`Release` com as flags pedidas.
struct Toque {
    tecla: CGKeyCode,
    flags: CGEventFlags,
}

fn toque_com_comando() -> Toque {
    Toque {
        tecla: KVK_ANSI_V,
        flags: CGEventFlags::CGEventFlagCommand,
    }
}

fn toque_simples(tecla: CGKeyCode) -> Toque {
    Toque {
        tecla,
        flags: CGEventFlags::CGEventFlagNull,
    }
}

fn enviar(toque: &Toque) -> Result<(), String> {
    // SAFETY: função do sistema sem argumentos, só consulta.
    if !unsafe { AXIsProcessTrusted() } {
        return Err("sem permissão de Acessibilidade para este processo".into());
    }
    let fonte = CGEventSource::new(CGEventSourceStateID::HIDSystemState)
        .map_err(|()| "falha ao criar a fonte de eventos".to_string())?;
    for pressionada in [true, false] {
        let evento = CGEvent::new_keyboard_event(fonte.clone(), toque.tecla, pressionada)
            .map_err(|()| "falha ao criar o evento de teclado".to_string())?;
        evento.set_flags(toque.flags);
        evento.post(CGEventTapLocation::Session);
        if pressionada {
            std::thread::sleep(PAUSA_ENTRE_EVENTOS);
        }
    }
    Ok(())
}

pub fn colar() -> Result<(), String> {
    enviar(&toque_com_comando())
}

pub fn enter() -> Result<(), String> {
    enviar(&toque_simples(KVK_RETURN))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn colar_e_command_mais_v_no_mesmo_evento() {
        let t = toque_com_comando();
        assert_eq!(t.tecla, 9);
        assert!(t.flags.contains(CGEventFlags::CGEventFlagCommand));
    }

    #[test]
    fn enter_nao_leva_modificador() {
        let t = toque_simples(KVK_RETURN);
        assert_eq!(t.tecla, 36);
        assert!(t.flags.is_empty());
    }
}

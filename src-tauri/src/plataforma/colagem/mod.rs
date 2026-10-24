//! Teclas simuladas para colar (Cmd+V no Mac, Ctrl+V no Windows) e para o Enter final (RF-03).
//! No Mac, sem a permissão de Acessibilidade o `Enigo::new` falha e o chamador cai em "não colado".

use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[cfg(target_os = "macos")]
const TECLA_MODIFICADORA: Key = Key::Meta;
#[cfg(not(target_os = "macos"))]
const TECLA_MODIFICADORA: Key = Key::Control;

// Códigos físicos da tecla V: independem do layout do teclado.
#[cfg(target_os = "macos")]
const TECLA_V: Key = Key::Other(9); // kVK_ANSI_V
#[cfg(not(target_os = "macos"))]
const TECLA_V: Key = Key::Other(0x56); // VK_V

/// Texto do atalho de colagem manual, para o aviso do gravador.
pub const fn atalho_manual() -> &'static str {
    if cfg!(target_os = "macos") {
        "Cmd+V"
    } else {
        "Ctrl+V"
    }
}

fn teclado() -> Result<Enigo, String> {
    let ajustes = Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    };
    Enigo::new(&ajustes).map_err(|e| e.to_string())
}

pub fn simular_colar() -> Result<(), String> {
    let mut e = teclado()?;
    let erro = |x: enigo::InputError| x.to_string();
    e.key(TECLA_MODIFICADORA, Direction::Press).map_err(erro)?;
    let colou = e.key(TECLA_V, Direction::Click).map_err(erro);
    // Solta o modificador mesmo se o V falhou, para não deixá-lo preso.
    let soltou = e.key(TECLA_MODIFICADORA, Direction::Release).map_err(erro);
    colou.and(soltou)
}

pub fn simular_enter() -> Result<(), String> {
    teclado()?
        .key(Key::Return, Direction::Click)
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod testes {
    #[test]
    fn atalho_manual_e_o_da_plataforma() {
        let esperado = if cfg!(target_os = "macos") {
            "Cmd+V"
        } else {
            "Ctrl+V"
        };
        assert_eq!(super::atalho_manual(), esperado);
    }
}

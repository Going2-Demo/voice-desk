//! Teclas simuladas para colar (Cmd+V no Mac, Ctrl+V no Windows) e para o Enter final (RF-03).
//! No Mac as teclas vão por CGEvent (`macos.rs`), com o Command marcado no próprio evento da tecla
//! V; sem a permissão de Acessibilidade o chamador cai em "não colado". No Windows, por enigo.

#[cfg(not(target_os = "macos"))]
use enigo::{Direction, Enigo, Key, Keyboard, Settings};

#[cfg(target_os = "macos")]
mod macos;

// Código físico da tecla V: independe do layout do teclado.
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

#[cfg(not(target_os = "macos"))]
fn teclado() -> Result<Enigo, String> {
    let ajustes = Settings {
        open_prompt_to_get_permissions: false,
        ..Settings::default()
    };
    Enigo::new(&ajustes).map_err(|e| e.to_string())
}

#[cfg(target_os = "macos")]
pub fn simular_colar() -> Result<(), String> {
    macos::colar()
}

#[cfg(not(target_os = "macos"))]
pub fn simular_colar() -> Result<(), String> {
    let mut e = teclado()?;
    let erro = |x: enigo::InputError| x.to_string();
    e.key(Key::Control, Direction::Press).map_err(erro)?;
    let colou = e.key(TECLA_V, Direction::Click).map_err(erro);
    // Solta o modificador mesmo se o V falhou, para não deixá-lo preso.
    let soltou = e.key(Key::Control, Direction::Release).map_err(erro);
    colou.and(soltou)
}

#[cfg(target_os = "macos")]
pub fn simular_enter() -> Result<(), String> {
    macos::enter()
}

#[cfg(not(target_os = "macos"))]
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

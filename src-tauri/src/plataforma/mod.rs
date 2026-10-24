//! Todo código específico de Mac ou Windows fica aqui, em `plataforma/<capacidade>/{mod.rs,
//! macos.rs, windows.rs}`. As capacidades (colagem, foco, seleção, teclado, mídia, permissões)
//! nascem nas histórias de cada uma; fora desta pasta não há `cfg(target_os)`.

/// Plataforma em que o app roda, no formato do banco (`modo_programa.plataforma`).
pub const fn atual() -> &'static str {
    if cfg!(target_os = "macos") {
        "macos"
    } else {
        "windows"
    }
}

#[cfg(test)]
mod testes {
    #[test]
    fn plataforma_atual_e_valida_no_banco() {
        assert!(["macos", "windows"].contains(&super::atual()));
    }
}

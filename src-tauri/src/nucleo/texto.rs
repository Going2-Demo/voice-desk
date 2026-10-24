//! Normalização de texto para comparações "sem diferenciar maiúsculas" (arquitetura, seção 8).

use unicode_normalization::UnicodeNormalization;

/// Minúsculas, Unicode NFC, espaços aparados. Acentos contam ("é" ≠ "e").
pub fn normalizar(texto: &str) -> String {
    texto.trim().nfc().collect::<String>().to_lowercase()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn minusculas_e_apara() {
        assert_eq!(normalizar("  Olá Mundo "), "olá mundo");
    }

    #[test]
    fn acento_conta() {
        assert_ne!(normalizar("é"), normalizar("e"));
    }

    #[test]
    fn forma_decomposta_vira_composta() {
        assert_eq!(normalizar("e\u{0301}"), normalizar("\u{00e9}"));
    }
}

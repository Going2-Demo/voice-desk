//! Erro único dos comandos: `{ codigo, mensagem, campo? }` (arquitetura, seção 5).

use serde::Serialize;
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct ErroApp {
    pub codigo: String,
    pub mensagem: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub campo: Option<String>,
}

impl ErroApp {
    pub fn novo(codigo: &str, mensagem: &str) -> Self {
        Self {
            codigo: codigo.into(),
            mensagem: mensagem.into(),
            campo: None,
        }
    }

    pub fn com_campo(mut self, campo: &str) -> Self {
        self.campo = Some(campo.into());
        self
    }

    pub fn interno(detalhe: impl std::fmt::Display) -> Self {
        // O detalhe vai para o log; a interface só recebe a mensagem genérica.
        tracing::error!(evento = "erro_interno", detalhe = %detalhe);
        Self::novo(
            "erro_interno",
            "Algo deu errado. Tente de novo; se repetir, veja os logs em Sobre.",
        )
    }
}

impl std::fmt::Display for ErroApp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.codigo, self.mensagem)
    }
}

impl std::error::Error for ErroApp {}

impl From<rusqlite::Error> for ErroApp {
    fn from(e: rusqlite::Error) -> Self {
        Self::interno(e)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn serializa_sem_campo_quando_nao_ha() {
        let json = serde_json::to_value(ErroApp::novo("x", "Mensagem")).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "codigo": "x", "mensagem": "Mensagem" })
        );
    }

    #[test]
    fn serializa_campo_de_validacao() {
        let json = serde_json::to_value(
            ErroApp::novo("modo_nome_duplicado", "Já existe.").com_campo("nome"),
        )
        .unwrap();
        assert_eq!(json["campo"], "nome");
    }

    #[test]
    fn erro_interno_nao_vaza_detalhe() {
        let e = ErroApp::interno("senha=123 no caminho /x");
        assert_eq!(e.codigo, "erro_interno");
        assert!(!e.mensagem.contains("senha"));
    }
}

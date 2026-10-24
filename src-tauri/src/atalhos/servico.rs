//! Leitura e gravação do atalho global do ditado na tabela local `atalho` (escopo `global`).

use super::validacao;
use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension, params};

pub const ID_DITADO: &str = "ditado";
const ESCOPO_GLOBAL: &str = "global";

/// Atalho sugerido no primeiro uso, diferente em cada sistema (definido pela Nina).
pub fn padrao(plataforma: &str) -> &'static str {
    if plataforma == "macos" {
        "Alt+Shift+Space"
    } else {
        "Ctrl+Alt+Space"
    }
}

pub fn obter(conexao: &Connection, plataforma: &str) -> Result<String, ErroApp> {
    let salvo: Option<String> = conexao
        .query_row(
            "SELECT combinacao FROM atalho WHERE id = ?1",
            [ID_DITADO],
            |l| l.get(0),
        )
        .optional()?;
    Ok(salvo.unwrap_or_else(|| padrao(plataforma).to_string()))
}

/// Valida o formato e confere se outra ação do app já usa a combinação (RF-01.1).
/// Devolve a combinação canônica; não grava nada.
pub fn validar_para_salvar(
    conexao: &Connection,
    plataforma: &str,
    bruto: &str,
) -> Result<String, ErroApp> {
    let canonica = validacao::validar(bruto, plataforma)?;
    let em_uso: Option<String> = conexao
        .query_row(
            "SELECT id FROM atalho WHERE combinacao = ?1 AND id <> ?2",
            params![canonica, ID_DITADO],
            |l| l.get(0),
        )
        .optional()?;
    if em_uso.is_some() {
        return Err(ErroApp::novo(
            "atalho_em_uso",
            &format!(
                "{} já é usado por outra ação do Voice Desk; escolha outra combinação.",
                validacao::exibir(&canonica, plataforma)
            ),
        )
        .com_campo("combinacao"));
    }
    Ok(canonica)
}

/// Grava uma combinação já validada.
pub fn gravar(conexao: &Connection, canonica: &str) -> Result<(), ErroApp> {
    conexao.execute(
        "INSERT INTO atalho (id, escopo, combinacao) VALUES (?1, ?2, ?3)
         ON CONFLICT (id) DO UPDATE SET combinacao = excluded.combinacao",
        params![ID_DITADO, ESCOPO_GLOBAL, canonica],
    )?;
    Ok(())
}

/// Valida e grava. Devolve a combinação canônica salva.
pub fn salvar(conexao: &Connection, plataforma: &str, bruto: &str) -> Result<String, ErroApp> {
    let canonica = validar_para_salvar(conexao, plataforma, bruto)?;
    gravar(conexao, &canonica)?;
    Ok(canonica)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco;

    fn banco_novo() -> (tempfile::TempDir, Connection) {
        let pasta = tempfile::tempdir().unwrap();
        let c = banco::abrir(&pasta.path().join("t.db")).unwrap();
        (pasta, c)
    }

    #[test]
    fn sem_atalho_salvo_usa_o_padrao_da_plataforma() {
        let (_p, c) = banco_novo();
        assert_eq!(obter(&c, "macos").unwrap(), "Alt+Shift+Space");
        assert_eq!(obter(&c, "windows").unwrap(), "Ctrl+Alt+Space");
    }

    #[test]
    fn salva_a_forma_canonica_e_le_de_volta() {
        let (_p, c) = banco_novo();
        assert_eq!(salvar(&c, "windows", "alt+ctrl+d").unwrap(), "Ctrl+Alt+D");
        assert_eq!(obter(&c, "windows").unwrap(), "Ctrl+Alt+D");
        salvar(&c, "windows", "Ctrl+Shift+F9").unwrap();
        assert_eq!(obter(&c, "windows").unwrap(), "Ctrl+Shift+F9");
    }

    #[test]
    fn recusa_invalido_sem_alterar_o_salvo() {
        let (_p, c) = banco_novo();
        salvar(&c, "windows", "Ctrl+Alt+D").unwrap();
        assert!(salvar(&c, "windows", "D").is_err());
        assert_eq!(obter(&c, "windows").unwrap(), "Ctrl+Alt+D");
    }

    #[test]
    fn recusa_combinacao_usada_por_outra_acao() {
        let (_p, c) = banco_novo();
        c.execute(
            "INSERT INTO atalho (id, escopo, combinacao) VALUES ('modo-x', 'modo:x', 'Ctrl+Alt+E')",
            [],
        )
        .unwrap();
        let e = salvar(&c, "windows", "Ctrl+Alt+E").unwrap_err();
        assert_eq!(e.codigo, "atalho_em_uso");
    }
}

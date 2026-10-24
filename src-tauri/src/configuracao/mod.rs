//! Configuração sincronizável. Toda escrita nas tabelas sincronizáveis passa por `gravar()` e
//! `excluir()`, que preenchem `atualizado_em`/`atualizado_por` e avisam a sincronização.

pub mod secoes;

use crate::nucleo::{ErroApp, relogio::Relogio};
use rusqlite::{Connection, types::Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tabela {
    Modo,
    ModoPrograma,
    PalavraGatilho,
    Vocabulario,
    Substituicao,
    PalavraPreenchimento,
    PreferenciaSincronizada,
}

impl Tabela {
    pub const fn nome(self) -> &'static str {
        match self {
            Self::Modo => "modo",
            Self::ModoPrograma => "modo_programa",
            Self::PalavraGatilho => "palavra_gatilho",
            Self::Vocabulario => "vocabulario",
            Self::Substituicao => "substituicao",
            Self::PalavraPreenchimento => "palavra_preenchimento",
            Self::PreferenciaSincronizada => "preferencia_sincronizada",
        }
    }
}

fn coluna_valida(c: &str) -> bool {
    !c.is_empty() && c.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

/// Insere ou atualiza a linha `id`. `colunas` são só as do domínio; as comuns são preenchidas aqui.
/// Devolve o carimbo gravado em `atualizado_em`.
pub fn gravar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    tabela: Tabela,
    id: &str,
    colunas: &[(&str, Value)],
) -> Result<String, ErroApp> {
    if colunas.iter().any(|(c, _)| !coluna_valida(c)) {
        return Err(ErroApp::interno("nome de coluna inválido em gravar()"));
    }
    let carimbo = relogio.proximo();
    let mut nomes = vec!["id", "atualizado_em", "atualizado_por"];
    let mut valores = vec![
        Value::from(id.to_string()),
        carimbo.clone().into(),
        maquina.to_string().into(),
    ];
    for (c, v) in colunas {
        nomes.push(c);
        valores.push(v.clone());
    }
    let marcadores: Vec<String> = (1..=nomes.len()).map(|i| format!("?{i}")).collect();
    let atualizacoes: Vec<String> = nomes
        .iter()
        .skip(1)
        .map(|c| format!("{c} = excluded.{c}"))
        .collect();
    let sql = format!(
        "INSERT INTO {t} ({cols}) VALUES ({vals}) ON CONFLICT (id) DO UPDATE SET {upd}, excluido_em = NULL",
        t = tabela.nome(),
        cols = nomes.join(", "),
        vals = marcadores.join(", "),
        upd = atualizacoes.join(", ")
    );
    conexao.execute(&sql, rusqlite::params_from_iter(valores))?;
    marcar_pendente(conexao)?;
    Ok(carimbo)
}

/// Exclusão lógica: a sincronização precisa saber que o item foi apagado.
pub fn excluir(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    tabela: Tabela,
    id: &str,
) -> Result<bool, ErroApp> {
    let carimbo = relogio.proximo();
    let sql = format!(
        "UPDATE {} SET excluido_em = ?1, atualizado_em = ?1, atualizado_por = ?2 WHERE id = ?3 AND excluido_em IS NULL",
        tabela.nome()
    );
    let n = conexao.execute(&sql, rusqlite::params![carimbo, maquina, id])?;
    if n > 0 {
        marcar_pendente(conexao)?;
    }
    Ok(n > 0)
}

/// Avisa a sincronização: há alteração local ainda não escrita na pasta de nuvem.
fn marcar_pendente(conexao: &Connection) -> Result<(), ErroApp> {
    conexao.execute("UPDATE sincronizacao_estado SET pendente = 1", [])?;
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco;

    fn base() -> (Connection, Relogio) {
        let mut c = Connection::open_in_memory().unwrap();
        banco::migrar(&mut c, banco::MIGRACOES).unwrap();
        (c, Relogio::novo("m1"))
    }

    fn vocab(c: &Connection, r: &Relogio, id: &str, palavra: &str) -> String {
        gravar(
            c,
            r,
            "m1",
            Tabela::Vocabulario,
            id,
            &[
                ("palavra", palavra.to_string().into()),
                (
                    "palavra_normalizada",
                    crate::nucleo::normalizar(palavra).into(),
                ),
            ],
        )
        .unwrap()
    }

    #[test]
    fn preenche_as_colunas_comuns() {
        let (c, r) = base();
        let carimbo = vocab(&c, &r, "a", "Going2");
        let (em, por, excl): (String, String, Option<String>) = c
            .query_row(
                "SELECT atualizado_em, atualizado_por, excluido_em FROM vocabulario WHERE id='a'",
                [],
                |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?)),
            )
            .unwrap();
        assert_eq!((em, por, excl), (carimbo, "m1".to_string(), None));
    }

    #[test]
    fn regravar_atualiza_e_avanca_o_carimbo() {
        let (c, r) = base();
        let a = vocab(&c, &r, "a", "Going2");
        let b = vocab(&c, &r, "a", "Going 2");
        assert!(b > a);
        let n: i64 = c
            .query_row("SELECT count(*) FROM vocabulario", [], |l| l.get(0))
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn excluir_e_logico_e_regravar_ressuscita() {
        let (c, r) = base();
        vocab(&c, &r, "a", "Going2");
        assert!(excluir(&c, &r, "m1", Tabela::Vocabulario, "a").unwrap());
        assert!(!excluir(&c, &r, "m1", Tabela::Vocabulario, "a").unwrap());
        let excl: Option<String> = c
            .query_row(
                "SELECT excluido_em FROM vocabulario WHERE id='a'",
                [],
                |l| l.get(0),
            )
            .unwrap();
        assert!(excl.is_some());
        vocab(&c, &r, "a", "Going2");
        let excl: Option<String> = c
            .query_row(
                "SELECT excluido_em FROM vocabulario WHERE id='a'",
                [],
                |l| l.get(0),
            )
            .unwrap();
        assert!(excl.is_none());
    }

    #[test]
    fn avisa_a_sincronizacao() {
        let (c, r) = base();
        c.execute("INSERT INTO sincronizacao_estado (dispositivo_id, nome_dispositivo) VALUES ('m1', 'Mac')", [])
            .unwrap();
        vocab(&c, &r, "a", "Going2");
        let p: i64 = c
            .query_row("SELECT pendente FROM sincronizacao_estado", [], |l| {
                l.get(0)
            })
            .unwrap();
        assert_eq!(p, 1);
    }

    #[test]
    fn recusa_nome_de_coluna_malicioso() {
        let (c, r) = base();
        let e = gravar(
            &c,
            &r,
            "m1",
            Tabela::Vocabulario,
            "a",
            &[("palavra; DROP TABLE modo", "x".to_string().into())],
        );
        assert_eq!(e.unwrap_err().codigo, "erro_interno");
    }
}

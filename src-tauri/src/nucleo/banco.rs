//! Abertura do SQLite e migrador próprio: aplica, em ordem pelo nome, os `.sql` embutidos que
//! ainda não constam em `_migracao`. Cada migração roda numa transação.

use super::ErroApp;
use rusqlite::{Connection, params};
use std::path::Path;

include!(concat!(env!("OUT_DIR"), "/migracoes.rs"));

pub fn abrir(caminho: &Path) -> Result<Connection, ErroApp> {
    let mut conexao = Connection::open(caminho)?;
    conexao.pragma_update(None, "journal_mode", "WAL")?;
    conexao.pragma_update(None, "foreign_keys", "ON")?;
    migrar(&mut conexao, MIGRACOES)?;
    Ok(conexao)
}

pub fn migrar(
    conexao: &mut Connection,
    migracoes: &[(&str, &str)],
) -> Result<Vec<String>, ErroApp> {
    conexao.execute_batch(
        "CREATE TABLE IF NOT EXISTS _migracao (nome TEXT PRIMARY KEY NOT NULL, aplicada_em TEXT NOT NULL);",
    )?;
    let mut aplicadas = Vec::new();
    for (nome, sql) in migracoes {
        let ja: bool = conexao.query_row(
            "SELECT EXISTS(SELECT 1 FROM _migracao WHERE nome = ?1)",
            [nome],
            |l| l.get(0),
        )?;
        if ja {
            continue;
        }
        let tx = conexao.transaction()?;
        tx.execute_batch(sql)?;
        tx.execute(
            "INSERT INTO _migracao (nome, aplicada_em) VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            params![nome],
        )?;
        tx.commit()?;
        tracing::info!(evento = "migracao_aplicada", nome = %nome);
        aplicadas.push((*nome).to_string());
    }
    Ok(aplicadas)
}

#[cfg(test)]
mod testes {
    use super::*;

    fn memoria() -> Connection {
        Connection::open_in_memory().unwrap()
    }

    #[test]
    fn aplica_o_esquema_inicial_e_registra() {
        let mut c = memoria();
        let feitas = migrar(&mut c, MIGRACOES).unwrap();
        assert_eq!(feitas.len(), MIGRACOES.len());
        let n: i64 = c
            .query_row("SELECT count(*) FROM _migracao", [], |l| l.get(0))
            .unwrap();
        assert_eq!(n as usize, MIGRACOES.len());
    }

    #[test]
    fn segunda_execucao_nao_reaplica() {
        let mut c = memoria();
        migrar(&mut c, MIGRACOES).unwrap();
        assert!(migrar(&mut c, MIGRACOES).unwrap().is_empty());
    }

    #[test]
    fn migracao_com_erro_nao_deixa_rastro() {
        let mut c = memoria();
        let ruins = [("1_ruim.sql", "CREATE TABLE a (x); CREATE TABLE a (x);")];
        assert!(migrar(&mut c, &ruins).is_err());
        let n: i64 = c
            .query_row("SELECT count(*) FROM _migracao", [], |l| l.get(0))
            .unwrap();
        assert_eq!(n, 0);
        let existe: i64 = c
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name = 'a'",
                [],
                |l| l.get(0),
            )
            .unwrap();
        assert_eq!(existe, 0);
    }

    #[test]
    fn todas_as_tabelas_da_arquitetura_existem() {
        let mut c = memoria();
        migrar(&mut c, MIGRACOES).unwrap();
        for t in [
            "modo",
            "modo_programa",
            "palavra_gatilho",
            "vocabulario",
            "substituicao",
            "palavra_preenchimento",
            "preferencia_sincronizada",
            "atalho",
            "preferencia_local",
            "historico_item",
            "historico_busca",
            "modelo_local",
            "sincronizacao_estado",
            "_migracao",
        ] {
            let n: i64 = c
                .query_row(
                    "SELECT count(*) FROM sqlite_master WHERE name = ?1",
                    [t],
                    |l| l.get(0),
                )
                .unwrap();
            assert_eq!(n, 1, "tabela ausente: {t}");
        }
    }

    #[test]
    fn nome_de_modo_e_unico_entre_nao_excluidos() {
        let mut c = memoria();
        migrar(&mut c, MIGRACOES).unwrap();
        let ins = "INSERT INTO modo (id, atualizado_em, atualizado_por, excluido_em, nome, nome_normalizado) VALUES (?1, 'a', 'm', ?2, 'Ditado', 'ditado')";
        c.execute(ins, params!["1", None::<String>]).unwrap();
        assert!(c.execute(ins, params!["2", None::<String>]).is_err());
        c.execute(ins, params!["3", Some("2026-01-01")]).unwrap();
    }

    #[test]
    fn busca_no_historico_acha_texto_inserido() {
        let mut c = memoria();
        migrar(&mut c, MIGRACOES).unwrap();
        c.execute(
            "INSERT INTO historico_item (id, criado_em, estado, texto_final) VALUES ('1', '2026-01-01T00:00:00.000Z', 'colado', 'reunião amanhã')",
            [],
        )
        .unwrap();
        let achou: i64 = c
            .query_row(
                "SELECT count(*) FROM historico_busca WHERE historico_busca MATCH 'amanhã'",
                [],
                |l| l.get(0),
            )
            .unwrap();
        assert_eq!(achou, 1);
    }

    #[test]
    fn abrir_cria_o_arquivo_migrado() {
        let pasta = tempfile::tempdir().unwrap();
        let c = abrir(&pasta.path().join("voice-desk.db")).unwrap();
        let n: i64 = c
            .query_row("SELECT count(*) FROM _migracao", [], |l| l.get(0))
            .unwrap();
        assert_eq!(n as usize, MIGRACOES.len());
    }
}

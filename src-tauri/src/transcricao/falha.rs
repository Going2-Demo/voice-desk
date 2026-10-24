//! Falha de transcrição no histórico (RF-02.3): o áudio fica guardado e o item aparece como
//! "falhou", com o motivo, sem colar nada. O texto ditado não existe nesse caminho.

use crate::audio::captura::Gravacao;
use crate::nucleo::ErroApp;
use crate::nucleo::caminhos::Caminhos;
use crate::transcricao::{ConfigTranscricao, flac};
use rusqlite::{Connection, OptionalExtension, params};

pub struct Falha<'a> {
    pub modo_id: Option<&'a str>,
    pub programa: Option<&'a str>,
    pub config: &'a ConfigTranscricao,
    pub gravacao: Option<&'a Gravacao>,
    pub erro: &'a ErroApp,
}

/// Grava o áudio em `audios/` (FLAC) e o item "falhou". Devolve o id do item.
pub fn registrar(
    conexao: &Connection,
    caminhos: &Caminhos,
    falha: &Falha<'_>,
) -> Result<String, ErroApp> {
    let id = uuid::Uuid::now_v7().to_string();
    let (arquivo, duracao_ms) = match falha.gravacao.filter(|g| !g.amostras.is_empty()) {
        Some(g) => {
            let pasta = caminhos.audios();
            std::fs::create_dir_all(&pasta).map_err(ErroApp::interno)?;
            let nome = format!("{id}.flac");
            std::fs::write(pasta.join(&nome), flac::codificar(g)?).map_err(ErroApp::interno)?;
            let ms = g.amostras.len() as u64 * 1000 / u64::from(g.taxa_hz.max(1));
            (Some(nome), ms as i64)
        }
        None => (None, 0),
    };
    let (nome, emoji): (String, String) = falha
        .modo_id
        .and_then(|m| {
            conexao
                .query_row("SELECT nome, emoji FROM modo WHERE id = ?1", [m], |l| {
                    Ok((l.get(0)?, l.get(1)?))
                })
                .optional()
                .ok()
                .flatten()
        })
        .unwrap_or_default();
    conexao.execute(
        "INSERT INTO historico_item (id, criado_em, modo_id, modo_nome, modo_emoji, programa_nome,
            duracao_audio_ms, audio_arquivo, estado, transcricao_provedor, transcricao_modelo,
            erro_codigo, erro_mensagem)
         VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'), ?2, ?3, ?4, ?5, ?6, ?7, 'falhou',
            ?8, ?9, ?10, ?11)",
        params![
            id,
            falha.modo_id,
            nome,
            emoji,
            falha.programa,
            duracao_ms,
            arquivo,
            falha.config.provedor.como_texto(),
            falha.config.modelo,
            falha.erro.codigo,
            falha.erro.mensagem,
        ],
    )?;
    tracing::info!(
        evento = "transcricao_falhou",
        provedor = falha.config.provedor.como_texto(),
        codigo = %falha.erro.codigo
    );
    Ok(id)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco::{MIGRACOES, migrar};
    use crate::transcricao::erro_sem_conexao;

    fn gravacao() -> Gravacao {
        Gravacao {
            taxa_hz: 16_000,
            amostras: vec![100; 16_000],
        }
    }

    #[test]
    fn guarda_audio_e_item_falhou_com_provedor_modelo_e_motivo() {
        let mut c = Connection::open_in_memory().unwrap();
        migrar(&mut c, MIGRACOES).unwrap();
        let pasta = tempfile::tempdir().unwrap();
        let caminhos = Caminhos::em(pasta.path());
        let cfg = ConfigTranscricao::default();
        let g = gravacao();
        let erro = erro_sem_conexao();
        let id = registrar(
            &c,
            &caminhos,
            &Falha {
                modo_id: None,
                programa: Some("Bloco de Notas"),
                config: &cfg,
                gravacao: Some(&g),
                erro: &erro,
            },
        )
        .unwrap();
        let (estado, prov, modelo, cod, arq, dur): (String, String, String, String, String, i64) = c
            .query_row(
                "SELECT estado, transcricao_provedor, transcricao_modelo, erro_codigo, audio_arquivo, duracao_audio_ms
                 FROM historico_item WHERE id = ?1",
                [&id],
                |l| Ok((l.get(0)?, l.get(1)?, l.get(2)?, l.get(3)?, l.get(4)?, l.get(5)?)),
            )
            .unwrap();
        assert_eq!(
            (
                estado.as_str(),
                prov.as_str(),
                modelo.as_str(),
                cod.as_str()
            ),
            ("falhou", "groq", "whisper-large-v3-turbo", "sem_conexao")
        );
        assert_eq!(dur, 1000);
        let bytes = std::fs::read(caminhos.audios().join(arq)).unwrap();
        assert_eq!(&bytes[..4], b"fLaC");
    }
}

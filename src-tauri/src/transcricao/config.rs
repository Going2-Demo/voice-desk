//! Provedor, modelo e idioma da transcrição por modo (RF-02.1 e RF-02.2). Lê e grava nas colunas
//! `idioma`, `transcricao_provedor` e `transcricao_modelo` da tabela `modo`; a escrita passa por
//! `configuracao::gravar()`.

use crate::configuracao::{self, Tabela};
use crate::nucleo::{ErroApp, relogio::Relogio};
use rusqlite::{Connection, OptionalExtension, params, types::Value};
use serde::{Deserialize, Serialize};
use specta::Type;

pub const MODELO_PADRAO: &str = "whisper-large-v3-turbo";
pub const MODELOS_GROQ: &[&str] = &["whisper-large-v3-turbo", "whisper-large-v3"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum ProvedorTranscricao {
    Groq,
    WhisperLocal,
}

impl ProvedorTranscricao {
    pub fn como_texto(self) -> &'static str {
        match self {
            Self::Groq => "groq",
            Self::WhisperLocal => "whisper_local",
        }
    }

    fn de_texto(t: &str) -> Self {
        if t == "whisper_local" {
            Self::WhisperLocal
        } else {
            Self::Groq
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub enum Idioma {
    #[serde(rename = "pt-BR")]
    PtBr,
    #[serde(rename = "en")]
    En,
    #[serde(rename = "auto")]
    Auto,
}

impl Idioma {
    pub fn como_texto(self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::En => "en",
            Self::Auto => "auto",
        }
    }

    fn de_texto(t: &str) -> Self {
        match t {
            "en" => Self::En,
            "auto" => Self::Auto,
            _ => Self::PtBr,
        }
    }

    /// Código ISO-639-1 enviado ao provedor; `None` na detecção automática.
    pub fn codigo_iso(self) -> Option<&'static str> {
        match self {
            Self::PtBr => Some("pt"),
            Self::En => Some("en"),
            Self::Auto => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ConfigTranscricao {
    pub provedor: ProvedorTranscricao,
    pub modelo: String,
    pub idioma: Idioma,
}

impl Default for ConfigTranscricao {
    fn default() -> Self {
        Self {
            provedor: ProvedorTranscricao::Groq,
            modelo: MODELO_PADRAO.into(),
            idioma: Idioma::PtBr,
        }
    }
}

fn modelo_ou_padrao(modelo: &str) -> String {
    if modelo.is_empty() {
        MODELO_PADRAO.into()
    } else {
        modelo.into()
    }
}

/// Configuração do modo `modo_id`; sem modo (ainda não há ativo) ou excluído, o padrão.
pub fn obter(conexao: &Connection, modo_id: Option<&str>) -> Result<ConfigTranscricao, ErroApp> {
    let linha = match modo_id {
        Some(id) => conexao
            .query_row(
                "SELECT transcricao_provedor, transcricao_modelo, idioma FROM modo
                 WHERE id = ?1 AND excluido_em IS NULL",
                [id],
                |l| {
                    Ok((
                        l.get::<_, String>(0)?,
                        l.get::<_, String>(1)?,
                        l.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?,
        None => None,
    };
    Ok(
        linha.map_or_else(ConfigTranscricao::default, |(p, m, i)| ConfigTranscricao {
            provedor: ProvedorTranscricao::de_texto(&p),
            modelo: modelo_ou_padrao(&m),
            idioma: Idioma::de_texto(&i),
        }),
    )
}

/// Modo ativo: o padrão escolhido no menu (ou o primeiro da lista), como em `bandeja::menu`.
pub fn modo_ativo(conexao: &Connection) -> Result<Option<String>, ErroApp> {
    Ok(crate::bandeja::menu::listar_modos(conexao)?
        .into_iter()
        .find(|m| m.padrao)
        .map(|m| m.id))
}

fn validar(config: &ConfigTranscricao) -> Result<(), ErroApp> {
    if config.provedor == ProvedorTranscricao::Groq
        && !MODELOS_GROQ.contains(&config.modelo.as_str())
    {
        return Err(
            ErroApp::novo("modelo_invalido", "Modelo desconhecido para o Groq.")
                .com_campo("modelo"),
        );
    }
    Ok(())
}

/// Grava provedor, modelo e idioma do modo (o modo precisa existir).
pub fn salvar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    modo_id: &str,
    config: &ConfigTranscricao,
) -> Result<ConfigTranscricao, ErroApp> {
    validar(config)?;
    // `gravar()` é um upsert: leva junto as colunas obrigatórias que não mudam.
    let nomes: Option<(String, String)> = conexao
        .query_row(
            "SELECT nome, nome_normalizado FROM modo WHERE id = ?1 AND excluido_em IS NULL",
            params![modo_id],
            |l| Ok((l.get(0)?, l.get(1)?)),
        )
        .optional()?;
    let Some((nome, normalizado)) = nomes else {
        return Err(
            ErroApp::novo("modo_inexistente", "Esse modo não existe mais.").com_campo("modo"),
        );
    };
    configuracao::gravar(
        conexao,
        relogio,
        maquina,
        Tabela::Modo,
        modo_id,
        &[
            ("nome", Value::from(nome)),
            ("nome_normalizado", Value::from(normalizado)),
            (
                "transcricao_provedor",
                Value::from(config.provedor.como_texto().to_string()),
            ),
            ("transcricao_modelo", Value::from(config.modelo.clone())),
            (
                "idioma",
                Value::from(config.idioma.como_texto().to_string()),
            ),
        ],
    )?;
    obter(conexao, Some(modo_id))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco::{MIGRACOES, migrar};

    fn banco() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        migrar(&mut c, MIGRACOES).unwrap();
        for (id, nome) in [("m1", "Ditado"), ("m2", "Inglês")] {
            c.execute(
                "INSERT INTO modo (id, atualizado_em, atualizado_por, nome, nome_normalizado)
                 VALUES (?1, 'a', 'm', ?2, lower(?2))",
                [id, nome],
            )
            .unwrap();
        }
        c
    }

    #[test]
    fn sem_modo_usa_groq_turbo_em_portugues() {
        let c = banco();
        let cfg = obter(&c, None).unwrap();
        assert_eq!(cfg, ConfigTranscricao::default());
        assert_eq!(obter(&c, Some("nao-existe")).unwrap(), cfg);
    }

    #[test]
    fn modos_guardam_provedor_modelo_e_idioma_cada_um() {
        let c = banco();
        let r = Relogio::novo("m");
        let en = ConfigTranscricao {
            provedor: ProvedorTranscricao::Groq,
            modelo: "whisper-large-v3".into(),
            idioma: Idioma::En,
        };
        salvar(&c, &r, "m", "m2", &en).unwrap();
        assert_eq!(obter(&c, Some("m2")).unwrap(), en);
        assert_eq!(obter(&c, Some("m1")).unwrap(), ConfigTranscricao::default());
    }

    #[test]
    fn idioma_automatico_nao_envia_codigo() {
        assert_eq!(Idioma::Auto.codigo_iso(), None);
        assert_eq!(Idioma::PtBr.codigo_iso(), Some("pt"));
        assert_eq!(Idioma::En.codigo_iso(), Some("en"));
    }

    #[test]
    fn rejeita_modelo_desconhecido_e_modo_inexistente() {
        let c = banco();
        let r = Relogio::novo("m");
        let ruim = ConfigTranscricao {
            modelo: "gpt-9".into(),
            ..Default::default()
        };
        assert_eq!(
            salvar(&c, &r, "m", "m1", &ruim).unwrap_err().codigo,
            "modelo_invalido"
        );
        let ok = ConfigTranscricao::default();
        assert_eq!(
            salvar(&c, &r, "m", "x", &ok).unwrap_err().codigo,
            "modo_inexistente"
        );
    }

    #[test]
    fn modelo_vazio_vira_o_padrao() {
        let c = banco();
        // 'transcricao_modelo' nasce vazio na tabela.
        assert_eq!(obter(&c, Some("m1")).unwrap().modelo, MODELO_PADRAO);
    }
}

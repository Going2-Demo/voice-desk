//! Limpeza do texto (F-09): lista editável de palavras de preenchimento (sincronizável, escrita
//! só por `configuracao::gravar()`) e o bloco "Texto do ditado" (idioma, remover preenchimento,
//! formatar, Enter depois de colar), gravado na linha do modo padrão. O texto ditado nunca vai
//! para o log.

pub mod algoritmo;

use crate::configuracao::{self, Tabela};
use crate::nucleo::{ErroApp, relogio::Relogio};
use crate::transcricao::config::{Idioma, modo_ativo};
use rusqlite::{Connection, OptionalExtension, params, types::Value};
use serde::{Deserialize, Serialize};
use specta::Type;

/// Máximo de palavras na lista.
pub const LIMITE_PALAVRAS: usize = 200;
/// Tamanho máximo de uma entrada, em caracteres.
pub const LIMITE_TAMANHO: usize = 40;
/// Tamanho máximo do texto da prévia, em caracteres.
pub const LIMITE_PREVIA: usize = 2000;
const NOME_MODO_PADRAO: &str = "Ditado";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct PalavraPreenchimento {
    pub id: String,
    pub palavra: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ListaPreenchimento {
    pub palavras: Vec<PalavraPreenchimento>,
    pub limite: u32,
}

/// Bloco "Texto do ditado" de Configurações › Geral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct OpcoesTexto {
    pub idioma: Idioma,
    pub remover_preenchimento: bool,
    pub formatar: bool,
    pub enviar_enter: bool,
}

impl Default for OpcoesTexto {
    /// Sem modo ainda: idioma pt-BR, limpeza e formatação ligadas, Enter desligado (design v3).
    fn default() -> Self {
        Self {
            idioma: Idioma::PtBr,
            remover_preenchimento: true,
            formatar: true,
            enviar_enter: false,
        }
    }
}

/// O que a etapa `limpar` precisa em cada ditado. O `Default` não faz nada.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConfigLimpeza {
    pub remover_preenchimento: bool,
    pub formatar: bool,
    pub palavras: Vec<String>,
}

fn limpar_entrada(palavra: &str) -> Result<(String, String), ErroApp> {
    let norm = algoritmo::normalizar_entrada(palavra);
    if norm.trim_matches('…').trim().is_empty() {
        return Err(
            ErroApp::novo("limpeza_vazia", "Digite uma palavra ou expressão.").com_campo("palavra"),
        );
    }
    if norm.chars().count() > LIMITE_TAMANHO {
        return Err(ErroApp::novo(
            "limpeza_palavra_longa",
            "Use até 40 caracteres por palavra.",
        )
        .com_campo("palavra"));
    }
    let exibida = palavra
        .replace("...", "…")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    Ok((exibida, norm))
}

fn todas(conexao: &Connection) -> Result<Vec<PalavraPreenchimento>, ErroApp> {
    let mut q = conexao.prepare(
        "SELECT id, palavra FROM palavra_preenchimento WHERE excluido_em IS NULL
         ORDER BY palavra_normalizada",
    )?;
    let linhas = q
        .query_map([], |l| {
            Ok(PalavraPreenchimento {
                id: l.get(0)?,
                palavra: l.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(linhas)
}

pub fn listar(conexao: &Connection) -> Result<ListaPreenchimento, ErroApp> {
    Ok(ListaPreenchimento {
        palavras: todas(conexao)?,
        limite: LIMITE_PALAVRAS as u32,
    })
}

pub fn adicionar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    palavra: &str,
) -> Result<PalavraPreenchimento, ErroApp> {
    let (exibida, norm) = limpar_entrada(palavra)?;
    let existente: Option<String> = conexao
        .query_row(
            "SELECT id FROM palavra_preenchimento WHERE palavra_normalizada = ?1 AND excluido_em IS NULL",
            [&norm],
            |l| l.get(0),
        )
        .optional()?;
    if existente.is_some() {
        return Err(ErroApp::novo(
            "limpeza_duplicada",
            &format!("“{exibida}” já está na lista."),
        )
        .com_campo("palavra"));
    }
    if todas(conexao)?.len() >= LIMITE_PALAVRAS {
        return Err(ErroApp::novo("limpeza_limite", "Limite de 200 palavras.").com_campo("palavra"));
    }
    let id = uuid::Uuid::now_v7().to_string();
    configuracao::gravar(
        conexao,
        relogio,
        maquina,
        Tabela::PalavraPreenchimento,
        &id,
        &[
            ("palavra", Value::from(exibida.clone())),
            ("palavra_normalizada", Value::from(norm)),
        ],
    )?;
    Ok(PalavraPreenchimento {
        id,
        palavra: exibida,
    })
}

pub fn remover(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
) -> Result<(), ErroApp> {
    configuracao::excluir(conexao, relogio, maquina, Tabela::PalavraPreenchimento, id)?;
    Ok(())
}

/// Opções do modo padrão; sem modo, as padrão do design.
pub fn opcoes_obter(conexao: &Connection) -> Result<OpcoesTexto, ErroApp> {
    let Some(id) = modo_ativo(conexao)? else {
        return Ok(OpcoesTexto::default());
    };
    opcoes_do_modo(conexao, &id)
}

/// Opções de um modo (o do ditado em curso); modo que sumiu devolve as padrão.
pub fn opcoes_do_modo(conexao: &Connection, modo_id: &str) -> Result<OpcoesTexto, ErroApp> {
    let linha = conexao
        .query_row(
            "SELECT idioma, remover_preenchimento, formatar, enviar_enter FROM modo
             WHERE id = ?1 AND excluido_em IS NULL",
            [modo_id],
            |l| {
                Ok((
                    l.get::<_, String>(0)?,
                    l.get::<_, i64>(1)?,
                    l.get::<_, i64>(2)?,
                    l.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    Ok(
        linha.map_or_else(OpcoesTexto::default, |(i, r, f, e)| OpcoesTexto {
            idioma: Idioma::de_texto(&i),
            remover_preenchimento: r != 0,
            formatar: f != 0,
            enviar_enter: e != 0,
        }),
    )
}

/// Grava as opções na linha do modo padrão; se ainda não há modo, cria o "Ditado".
pub fn opcoes_salvar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    opcoes: &OpcoesTexto,
) -> Result<OpcoesTexto, ErroApp> {
    let id = match modo_ativo(conexao)? {
        Some(id) => id,
        None => uuid::Uuid::now_v7().to_string(),
    };
    let nomes: Option<(String, String)> = conexao
        .query_row(
            "SELECT nome, nome_normalizado FROM modo WHERE id = ?1 AND excluido_em IS NULL",
            params![id],
            |l| Ok((l.get(0)?, l.get(1)?)),
        )
        .optional()?;
    let (nome, normalizado) = nomes.unwrap_or_else(|| {
        (
            NOME_MODO_PADRAO.to_string(),
            crate::nucleo::normalizar(NOME_MODO_PADRAO),
        )
    });
    configuracao::gravar(
        conexao,
        relogio,
        maquina,
        Tabela::Modo,
        &id,
        &[
            ("nome", Value::from(nome)),
            ("nome_normalizado", Value::from(normalizado)),
            (
                "idioma",
                Value::from(opcoes.idioma.como_texto().to_string()),
            ),
            (
                "remover_preenchimento",
                Value::from(i64::from(opcoes.remover_preenchimento)),
            ),
            ("formatar", Value::from(i64::from(opcoes.formatar))),
            ("enviar_enter", Value::from(i64::from(opcoes.enviar_enter))),
        ],
    )?;
    opcoes_do_modo(conexao, &id)
}

/// Limpeza e lista que valem para o ditado do modo `modo_id`.
pub fn config_do_modo(
    conexao: &Connection,
    modo_id: Option<&str>,
) -> Result<(ConfigLimpeza, OpcoesTexto), ErroApp> {
    let opcoes = match modo_id {
        Some(id) => opcoes_do_modo(conexao, id)?,
        None => OpcoesTexto::default(),
    };
    let palavras = todas(conexao)?.into_iter().map(|p| p.palavra).collect();
    Ok((
        ConfigLimpeza {
            remover_preenchimento: opcoes.remover_preenchimento,
            formatar: opcoes.formatar,
            palavras,
        },
        opcoes,
    ))
}

/// Prévia ao vivo da tela: o mesmo caminho do ditado, com as opções e a lista salvas.
pub fn previa(conexao: &Connection, texto: &str) -> Result<String, ErroApp> {
    if texto.chars().count() > LIMITE_PREVIA {
        return Err(
            ErroApp::novo("limpeza_texto_longo", "Use até 2.000 caracteres no teste.")
                .com_campo("texto"),
        );
    }
    let modo = modo_ativo(conexao)?;
    let (config, _) = config_do_modo(conexao, modo.as_deref())?;
    Ok(algoritmo::limpar(
        texto,
        &config.palavras,
        config.remover_preenchimento,
        config.formatar,
    ))
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

    fn nomes(c: &Connection) -> Vec<String> {
        listar(c)
            .unwrap()
            .palavras
            .into_iter()
            .map(|p| p.palavra)
            .collect()
    }

    #[test]
    fn lista_vem_preenchida_em_portugues_e_ingles() {
        let (c, _) = base();
        let n = nomes(&c);
        for esperado in ["é…", "hã", "né", "tipo", "uh", "you know"] {
            assert!(n.iter().any(|p| p == esperado), "faltou {esperado}");
        }
    }

    #[test]
    fn adicionar_aparece_e_duplicada_e_recusada_sem_diferenciar_caixa() {
        let (c, r) = base();
        let p = adicionar(&c, &r, "m1", "  Basicamente ").unwrap();
        assert_eq!(p.palavra, "Basicamente");
        assert!(nomes(&c).contains(&"Basicamente".to_string()));
        let e = adicionar(&c, &r, "m1", "basicamente").unwrap_err();
        assert_eq!(e.codigo, "limpeza_duplicada");
        assert_eq!(e.campo.as_deref(), Some("palavra"));
        assert_eq!(
            adicionar(&c, &r, "m1", "TIPO").unwrap_err().codigo,
            "limpeza_duplicada"
        );
    }

    #[test]
    fn vazia_e_comprida_sao_recusadas_e_tres_pontos_viram_reticencias() {
        let (c, r) = base();
        assert_eq!(
            adicionar(&c, &r, "m1", "  ").unwrap_err().codigo,
            "limpeza_vazia"
        );
        assert_eq!(
            adicionar(&c, &r, "m1", "...").unwrap_err().codigo,
            "limpeza_vazia"
        );
        let longa = "a".repeat(LIMITE_TAMANHO + 1);
        assert_eq!(
            adicionar(&c, &r, "m1", &longa).unwrap_err().codigo,
            "limpeza_palavra_longa"
        );
        assert_eq!(adicionar(&c, &r, "m1", "ahn...").unwrap().palavra, "ahn…");
    }

    #[test]
    fn limite_de_200_palavras() {
        let (c, r) = base();
        while listar(&c).unwrap().palavras.len() < LIMITE_PALAVRAS {
            let n = listar(&c).unwrap().palavras.len();
            adicionar(&c, &r, "m1", &format!("muleta{n}")).unwrap();
        }
        let e = adicionar(&c, &r, "m1", "uma a mais").unwrap_err();
        assert_eq!(e.codigo, "limpeza_limite");
    }

    #[test]
    fn remover_tira_da_lista_e_libera_o_nome() {
        let (c, r) = base();
        let p = adicionar(&c, &r, "m1", "basicamente").unwrap();
        remover(&c, &r, "m1", &p.id).unwrap();
        assert!(!nomes(&c).contains(&"basicamente".to_string()));
        adicionar(&c, &r, "m1", "basicamente").unwrap();
    }

    #[test]
    fn lista_esvaziada_continua_vazia() {
        let (c, r) = base();
        for p in listar(&c).unwrap().palavras {
            remover(&c, &r, "m1", &p.id).unwrap();
        }
        assert!(listar(&c).unwrap().palavras.is_empty());
    }

    #[test]
    fn sem_modo_vale_o_padrao_do_design() {
        let (c, _) = base();
        assert_eq!(opcoes_obter(&c).unwrap(), OpcoesTexto::default());
    }

    #[test]
    fn salvar_sem_modo_cria_o_ditado_e_guarda_cada_opcao() {
        let (c, r) = base();
        let o = OpcoesTexto {
            idioma: Idioma::En,
            remover_preenchimento: false,
            formatar: true,
            enviar_enter: true,
        };
        assert_eq!(opcoes_salvar(&c, &r, "m1", &o).unwrap(), o);
        assert_eq!(opcoes_obter(&c).unwrap(), o);
        let nome: String = c
            .query_row("SELECT nome FROM modo WHERE excluido_em IS NULL", [], |l| {
                l.get(0)
            })
            .unwrap();
        assert_eq!(nome, "Ditado");
        // segunda gravação atualiza a mesma linha
        let o2 = OpcoesTexto {
            remover_preenchimento: true,
            ..o
        };
        opcoes_salvar(&c, &r, "m1", &o2).unwrap();
        let n: i64 = c
            .query_row(
                "SELECT count(*) FROM modo WHERE excluido_em IS NULL",
                [],
                |l| l.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
        assert_eq!(opcoes_obter(&c).unwrap(), o2);
    }

    #[test]
    fn ligar_e_desligar_so_mexe_nas_proprias_colunas() {
        let (c, r) = base();
        let ligado = OpcoesTexto::default();
        opcoes_salvar(&c, &r, "m1", &ligado).unwrap();
        c.execute(
            "UPDATE modo SET transcricao_modelo = 'whisper-large-v3'",
            [],
        )
        .unwrap();
        opcoes_salvar(
            &c,
            &r,
            "m1",
            &OpcoesTexto {
                formatar: false,
                ..ligado
            },
        )
        .unwrap();
        let m: String = c
            .query_row("SELECT transcricao_modelo FROM modo", [], |l| l.get(0))
            .unwrap();
        assert_eq!(m, "whisper-large-v3");
    }

    #[test]
    fn previa_segue_as_opcoes_salvas() {
        let (c, r) = base();
        assert_eq!(
            previa(&c, "hã… chego em dez minutos").unwrap(),
            "Chego em dez minutos."
        );
        let desligado = OpcoesTexto {
            remover_preenchimento: false,
            formatar: false,
            ..OpcoesTexto::default()
        };
        opcoes_salvar(&c, &r, "m1", &desligado).unwrap();
        assert_eq!(
            previa(&c, "hã… chego em dez minutos").unwrap(),
            "hã… chego em dez minutos"
        );
        let longo = "a".repeat(LIMITE_PREVIA + 1);
        assert_eq!(
            previa(&c, &longo).unwrap_err().codigo,
            "limpeza_texto_longo"
        );
    }

    #[test]
    fn config_do_modo_leva_a_lista_editada() {
        let (c, r) = base();
        adicionar(&c, &r, "m1", "basicamente").unwrap();
        let (cfg, _) = config_do_modo(&c, None).unwrap();
        assert!(cfg.palavras.contains(&"basicamente".to_string()));
        assert!(cfg.remover_preenchimento && cfg.formatar);
    }
}

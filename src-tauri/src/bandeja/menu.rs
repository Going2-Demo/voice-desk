//! Modelo do menu do ícone (RF-05.1), sem depender do Tauri para ser testável, e a leitura/escrita
//! do modo padrão (`preferencia_sincronizada.modo_padrao_id`).

use super::estado::EstadoIcone;
use crate::configuracao::{self, Tabela};
use crate::nucleo::{ErroApp, relogio::Relogio};
use rusqlite::{Connection, OptionalExtension};

pub const ID_DITADO: &str = "ditado";
pub const ID_VOCABULARIO: &str = "vocabulario";
pub const ID_HISTORICO: &str = "historico";
pub const ID_CONFIG: &str = "config";
pub const ID_SAIR: &str = "sair";
pub const PREFIXO_MODO: &str = "modo:";
const CHAVE_MODO_PADRAO: &str = "modo_padrao_id";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModoMenu {
    pub id: String,
    pub rotulo: String,
    pub padrao: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItemModelo {
    Acao { id: String, rotulo: String },
    Modos(Vec<ModoMenu>),
    Separador,
}

pub fn modelo(estado: EstadoIcone, modos: Vec<ModoMenu>) -> Vec<ItemModelo> {
    let acao = |id: &str, rotulo: &str| ItemModelo::Acao {
        id: id.into(),
        rotulo: rotulo.into(),
    };
    let rotulo_ditado = if estado.gravando() {
        "Parar ditado"
    } else {
        "Iniciar ditado"
    };
    let mut itens = vec![acao(ID_DITADO, rotulo_ditado)];
    if !modos.is_empty() {
        itens.push(ItemModelo::Modos(modos));
    }
    itens.extend([
        ItemModelo::Separador,
        acao(ID_VOCABULARIO, "Adicionar palavra ao vocabulário…"),
        acao(ID_HISTORICO, "Abrir histórico"),
        acao(ID_CONFIG, "Abrir configurações"),
        ItemModelo::Separador,
        acao(ID_SAIR, "Sair do Voice Desk"),
    ]);
    itens
}

pub fn id_do_modo(id_item: &str) -> Option<&str> {
    id_item.strip_prefix(PREFIXO_MODO).filter(|i| !i.is_empty())
}

fn modo_padrao_id(conexao: &Connection) -> Result<Option<String>, ErroApp> {
    let json: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_sincronizada WHERE chave = ?1 AND excluido_em IS NULL",
            [CHAVE_MODO_PADRAO],
            |l| l.get(0),
        )
        .optional()?;
    Ok(json.and_then(|j| serde_json::from_str::<String>(&j).ok()))
}

/// Modos ativos, em ordem de nome; o padrão é o escolhido ou, na falta, o primeiro.
pub fn listar_modos(conexao: &Connection) -> Result<Vec<ModoMenu>, ErroApp> {
    let mut consulta = conexao.prepare(
        "SELECT id, emoji, nome FROM modo WHERE excluido_em IS NULL ORDER BY nome_normalizado",
    )?;
    let linhas = consulta
        .query_map([], |l| {
            Ok((
                l.get::<_, String>(0)?,
                l.get::<_, String>(1)?,
                l.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    let escolhido = modo_padrao_id(conexao)?;
    let padrao = linhas
        .iter()
        .find(|(id, ..)| Some(id) == escolhido.as_ref())
        .or_else(|| linhas.first())
        .map(|(id, ..)| id.clone());
    Ok(linhas
        .into_iter()
        .map(|(id, emoji, nome)| ModoMenu {
            rotulo: if emoji.is_empty() {
                nome
            } else {
                format!("{emoji} {nome}")
            },
            padrao: Some(&id) == padrao.as_ref(),
            id,
        })
        .collect())
}

/// Define o modo padrão (usado quando nenhum programa associado está em foco, F-12).
pub fn escolher_modo(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    modo_id: &str,
) -> Result<(), ErroApp> {
    let existe: bool = conexao.query_row(
        "SELECT EXISTS(SELECT 1 FROM modo WHERE id = ?1 AND excluido_em IS NULL)",
        [modo_id],
        |l| l.get(0),
    )?;
    if !existe {
        return Err(ErroApp::novo(
            "modo_inexistente",
            "Esse modo não existe mais. Abra o menu de novo.",
        ));
    }
    let id_preferencia: Option<String> = conexao
        .query_row(
            "SELECT id FROM preferencia_sincronizada WHERE chave = ?1 AND excluido_em IS NULL",
            [CHAVE_MODO_PADRAO],
            |l| l.get(0),
        )
        .optional()?;
    let id_preferencia = id_preferencia.unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    configuracao::gravar(
        conexao,
        relogio,
        maquina,
        Tabela::PreferenciaSincronizada,
        &id_preferencia,
        &[
            ("chave", CHAVE_MODO_PADRAO.to_string().into()),
            (
                "valor",
                serde_json::to_string(modo_id)
                    .map_err(ErroApp::interno)?
                    .into(),
            ),
        ],
    )?;
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

    fn modo(c: &Connection, r: &Relogio, id: &str, emoji: &str, nome: &str) {
        configuracao::gravar(
            c,
            r,
            "m1",
            Tabela::Modo,
            id,
            &[
                ("nome", nome.to_string().into()),
                ("nome_normalizado", crate::nucleo::normalizar(nome).into()),
                ("emoji", emoji.to_string().into()),
            ],
        )
        .unwrap();
    }

    fn rotulos(itens: &[ItemModelo]) -> Vec<String> {
        itens
            .iter()
            .filter_map(|i| match i {
                ItemModelo::Acao { rotulo, .. } => Some(rotulo.clone()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn menu_tem_os_itens_do_rf_05_1_e_a_caixa_rapida_do_rf_07_3() {
        let modos = vec![ModoMenu {
            id: "a".into(),
            rotulo: "Ditado".into(),
            padrao: true,
        }];
        let itens = modelo(EstadoIcone::Parado, modos);
        assert_eq!(
            rotulos(&itens),
            [
                "Iniciar ditado",
                "Adicionar palavra ao vocabulário…",
                "Abrir histórico",
                "Abrir configurações",
                "Sair do Voice Desk"
            ]
        );
        assert!(itens.iter().any(|i| matches!(i, ItemModelo::Modos(_))));
    }

    #[test]
    fn item_de_ditado_vira_parar_enquanto_grava() {
        assert_eq!(
            rotulos(&modelo(EstadoIcone::Gravando, vec![]))[0],
            "Parar ditado"
        );
        assert_eq!(
            rotulos(&modelo(EstadoIcone::Processando, vec![]))[0],
            "Iniciar ditado"
        );
    }

    #[test]
    fn id_do_modo_extrai_so_ids_validos() {
        assert_eq!(id_do_modo("modo:abc"), Some("abc"));
        assert_eq!(id_do_modo("modo:"), None);
        assert_eq!(id_do_modo("sair"), None);
    }

    #[test]
    fn sem_escolha_o_primeiro_modo_e_o_padrao() {
        let (c, r) = base();
        modo(&c, &r, "b", "✍️", "Mensagem");
        modo(&c, &r, "a", "", "Ditado");
        let modos = listar_modos(&c).unwrap();
        assert_eq!(
            modos.iter().map(|m| m.rotulo.as_str()).collect::<Vec<_>>(),
            ["Ditado", "✍️ Mensagem"]
        );
        assert_eq!(
            modos.iter().map(|m| m.padrao).collect::<Vec<_>>(),
            [true, false]
        );
    }

    #[test]
    fn escolher_modo_muda_o_padrao_e_repetir_atualiza_a_mesma_linha() {
        let (c, r) = base();
        modo(&c, &r, "a", "", "Ditado");
        modo(&c, &r, "b", "", "Mensagem");
        escolher_modo(&c, &r, "m1", "b").unwrap();
        assert!(listar_modos(&c).unwrap()[1].padrao);
        escolher_modo(&c, &r, "m1", "a").unwrap();
        assert!(listar_modos(&c).unwrap()[0].padrao);
        let n: i64 = c
            .query_row(
                "SELECT COUNT(*) FROM preferencia_sincronizada WHERE chave = 'modo_padrao_id'",
                [],
                |l| l.get(0),
            )
            .unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn modo_excluido_ou_inexistente_e_recusado() {
        let (c, r) = base();
        modo(&c, &r, "a", "", "Ditado");
        configuracao::excluir(&c, &r, "m1", Tabela::Modo, "a").unwrap();
        let e = escolher_modo(&c, &r, "m1", "a").unwrap_err();
        assert_eq!(e.codigo, "modo_inexistente");
        assert!(listar_modos(&c).unwrap().is_empty());
    }

    #[test]
    fn padrao_apontando_para_modo_excluido_cai_no_primeiro() {
        let (c, r) = base();
        modo(&c, &r, "a", "", "Ditado");
        modo(&c, &r, "b", "", "Mensagem");
        escolher_modo(&c, &r, "m1", "b").unwrap();
        configuracao::excluir(&c, &r, "m1", Tabela::Modo, "b").unwrap();
        assert!(listar_modos(&c).unwrap()[0].padrao);
    }
}

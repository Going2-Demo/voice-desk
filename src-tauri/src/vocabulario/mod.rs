//! Vocabulário personalizado (F-07): palavras, nomes e siglas que o provedor de transcrição deve
//! reconhecer. A lista é sincronizável (escrita só por `configuracao::gravar()`); a cada ditado
//! as palavras que cabem na dica vão no `prompt` do provedor (RF-07.2). O texto das palavras
//! nunca vai para o log.

use crate::configuracao::{self, Tabela};
use crate::nucleo::{ErroApp, relogio::Relogio};
use rusqlite::{Connection, OptionalExtension, types::Value};
use serde::Serialize;
use specta::Type;

/// Máximo de palavras no vocabulário (RF-07.4).
pub const LIMITE_PALAVRAS: usize = 500;
/// Tamanho máximo de uma palavra ou expressão, em caracteres.
pub const LIMITE_TAMANHO: usize = 80;
/// Orçamento da dica em caracteres: o provedor só considera cerca de 224 tokens (arquitetura,
/// seção 13); 500 caracteres ficam abaixo disso em português e inglês.
pub const LIMITE_DICA_CARACTERES: usize = 500;
const SEPARADOR_DICA: &str = ", ";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct Palavra {
    pub id: String,
    pub palavra: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct ListaVocabulario {
    /// Palavras que casam com a busca, as mais recentes primeiro.
    pub palavras: Vec<Palavra>,
    /// Total no vocabulário, sem considerar a busca.
    pub total: u32,
    pub limite: u32,
    /// Quantas palavras vão como dica a cada ditado (as mais recentes que cabem).
    pub na_dica: u32,
}

/// Forma comparável: sem espaços sobrando e sem diferenciar maiúsculas.
pub fn normalizar(palavra: &str) -> String {
    palavra
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn limpar(palavra: &str) -> Result<(String, String), ErroApp> {
    let limpa = palavra.split_whitespace().collect::<Vec<_>>().join(" ");
    if limpa.is_empty() {
        return Err(
            ErroApp::novo("vocabulario_vazio", "Digite uma palavra ou nome.").com_campo("palavra"),
        );
    }
    if limpa.chars().count() > LIMITE_TAMANHO {
        return Err(ErroApp::novo(
            "vocabulario_palavra_longa",
            "Use até 80 caracteres por palavra.",
        )
        .com_campo("palavra"));
    }
    let normalizada = normalizar(&limpa);
    Ok((limpa, normalizada))
}

fn contar(conexao: &Connection) -> Result<usize, ErroApp> {
    let n: i64 = conexao.query_row(
        "SELECT count(*) FROM vocabulario WHERE excluido_em IS NULL",
        [],
        |l| l.get(0),
    )?;
    Ok(usize::try_from(n).unwrap_or(0))
}

fn id_da_normalizada(conexao: &Connection, normalizada: &str) -> Result<Option<String>, ErroApp> {
    Ok(conexao
        .query_row(
            "SELECT id FROM vocabulario WHERE palavra_normalizada = ?1 AND excluido_em IS NULL",
            [normalizada],
            |l| l.get(0),
        )
        .optional()?)
}

fn erro_duplicada(palavra: &str) -> ErroApp {
    ErroApp::novo(
        "vocabulario_duplicada",
        &format!("“{palavra}” já está no vocabulário."),
    )
    .com_campo("palavra")
}

fn todas(conexao: &Connection) -> Result<Vec<Palavra>, ErroApp> {
    let mut q = conexao.prepare(
        "SELECT id, palavra FROM vocabulario WHERE excluido_em IS NULL
         ORDER BY atualizado_em DESC, id DESC",
    )?;
    let linhas = q
        .query_map([], |l| {
            Ok(Palavra {
                id: l.get(0)?,
                palavra: l.get(1)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(linhas)
}

/// Palavras que cabem no orçamento da dica, começando pelas mais recentes.
fn selecionar_para_dica(palavras: &[Palavra]) -> Vec<&str> {
    let mut usados = 0usize;
    let mut escolhidas = Vec::new();
    for p in palavras {
        let custo = p.palavra.chars().count()
            + if escolhidas.is_empty() {
                0
            } else {
                SEPARADOR_DICA.len()
            };
        if usados + custo > LIMITE_DICA_CARACTERES {
            break;
        }
        usados += custo;
        escolhidas.push(p.palavra.as_str());
    }
    escolhidas
}

/// Lista (filtrada pela busca, sem diferenciar maiúsculas) e contagens para a tela.
pub fn listar(conexao: &Connection, busca: Option<&str>) -> Result<ListaVocabulario, ErroApp> {
    let todas = todas(conexao)?;
    let na_dica = selecionar_para_dica(&todas).len();
    let total = todas.len();
    let termo = busca.map(normalizar).filter(|t| !t.is_empty());
    let palavras = match termo {
        Some(t) => todas
            .into_iter()
            .filter(|p| normalizar(&p.palavra).contains(&t))
            .collect(),
        None => todas,
    };
    Ok(ListaVocabulario {
        palavras,
        total: u32::try_from(total).unwrap_or(u32::MAX),
        limite: LIMITE_PALAVRAS as u32,
        na_dica: u32::try_from(na_dica).unwrap_or(u32::MAX),
    })
}

pub fn adicionar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    palavra: &str,
) -> Result<Palavra, ErroApp> {
    let (limpa, normalizada) = limpar(palavra)?;
    if id_da_normalizada(conexao, &normalizada)?.is_some() {
        return Err(erro_duplicada(&limpa));
    }
    if contar(conexao)? >= LIMITE_PALAVRAS {
        return Err(
            ErroApp::novo("vocabulario_limite", "Limite de 500 palavras.").com_campo("palavra"),
        );
    }
    let id = uuid::Uuid::now_v7().to_string();
    gravar_palavra(conexao, relogio, maquina, &id, &limpa, &normalizada)?;
    Ok(Palavra { id, palavra: limpa })
}

pub fn editar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
    palavra: &str,
) -> Result<Palavra, ErroApp> {
    let (limpa, normalizada) = limpar(palavra)?;
    let existe: Option<String> = conexao
        .query_row(
            "SELECT id FROM vocabulario WHERE id = ?1 AND excluido_em IS NULL",
            [id],
            |l| l.get(0),
        )
        .optional()?;
    if existe.is_none() {
        return Err(ErroApp::novo(
            "vocabulario_nao_encontrada",
            "Essa palavra não está mais no vocabulário.",
        ));
    }
    if id_da_normalizada(conexao, &normalizada)?.is_some_and(|outro| outro != id) {
        return Err(erro_duplicada(&limpa));
    }
    gravar_palavra(conexao, relogio, maquina, id, &limpa, &normalizada)?;
    Ok(Palavra {
        id: id.into(),
        palavra: limpa,
    })
}

pub fn remover(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
) -> Result<(), ErroApp> {
    configuracao::excluir(conexao, relogio, maquina, Tabela::Vocabulario, id)?;
    Ok(())
}

fn gravar_palavra(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
    palavra: &str,
    normalizada: &str,
) -> Result<(), ErroApp> {
    configuracao::gravar(
        conexao,
        relogio,
        maquina,
        Tabela::Vocabulario,
        id,
        &[
            ("palavra", Value::from(palavra.to_string())),
            ("palavra_normalizada", Value::from(normalizada.to_string())),
        ],
    )?;
    Ok(())
}

/// Dica enviada ao provedor a cada ditado (RF-07.2): as palavras mais recentes que cabem, em
/// uma frase separada por vírgulas. Vocabulário vazio devolve `None` e a transcrição segue sem
/// dica.
pub fn dica(conexao: &Connection) -> Result<Option<String>, ErroApp> {
    let todas = todas(conexao)?;
    let escolhidas = selecionar_para_dica(&todas);
    if escolhidas.is_empty() {
        return Ok(None);
    }
    Ok(Some(escolhidas.join(SEPARADOR_DICA)))
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

    fn add(c: &Connection, r: &Relogio, p: &str) -> Result<Palavra, ErroApp> {
        adicionar(c, r, "m1", p)
    }

    #[test]
    fn palavra_adicionada_aparece_na_lista_e_na_busca() {
        let (c, r) = base();
        add(&c, &r, "Going2").unwrap();
        add(&c, &r, "Hortolândia").unwrap();
        let l = listar(&c, None).unwrap();
        assert_eq!(l.total, 2);
        assert_eq!(
            l.palavras[0].palavra, "Hortolândia",
            "mais recente primeiro"
        );
        let b = listar(&c, Some("  GOING ")).unwrap();
        assert_eq!(b.palavras.len(), 1);
        assert_eq!(b.palavras[0].palavra, "Going2");
        assert_eq!(b.total, 2, "o total ignora a busca");
        assert!(listar(&c, Some("xyz")).unwrap().palavras.is_empty());
    }

    #[test]
    fn duplicada_sem_diferenciar_maiusculas_e_recusada() {
        let (c, r) = base();
        add(&c, &r, "Going2").unwrap();
        let e = add(&c, &r, "  gOING2 ").unwrap_err();
        assert_eq!(e.codigo, "vocabulario_duplicada");
        assert_eq!(e.campo.as_deref(), Some("palavra"));
        assert_eq!(listar(&c, None).unwrap().total, 1);
    }

    #[test]
    fn vazia_ou_so_espacos_e_recusada_e_comprida_tambem() {
        let (c, r) = base();
        assert_eq!(add(&c, &r, "   ").unwrap_err().codigo, "vocabulario_vazio");
        let longa = "a".repeat(LIMITE_TAMANHO + 1);
        assert_eq!(
            add(&c, &r, &longa).unwrap_err().codigo,
            "vocabulario_palavra_longa"
        );
        add(&c, &r, &"a".repeat(LIMITE_TAMANHO)).unwrap();
    }

    #[test]
    fn a_501a_palavra_e_recusada_com_a_mensagem_do_limite() {
        let (c, r) = base();
        for i in 0..LIMITE_PALAVRAS {
            add(&c, &r, &format!("palavra{i}")).unwrap();
        }
        let e = add(&c, &r, "mais uma").unwrap_err();
        assert_eq!(e.codigo, "vocabulario_limite");
        assert_eq!(e.mensagem, "Limite de 500 palavras.");
        assert_eq!(listar(&c, None).unwrap().total, 500);
    }

    #[test]
    fn remover_libera_espaco_e_o_nome_para_nova_inclusao() {
        let (c, r) = base();
        let p = add(&c, &r, "SLA").unwrap();
        remover(&c, &r, "m1", &p.id).unwrap();
        assert_eq!(listar(&c, None).unwrap().total, 0);
        add(&c, &r, "sla").unwrap();
        assert_eq!(listar(&c, None).unwrap().total, 1);
    }

    #[test]
    fn editar_troca_o_texto_e_barra_duplicata_de_outra_palavra() {
        let (c, r) = base();
        let a = add(&c, &r, "Grafana").unwrap();
        add(&c, &r, "Dokploy").unwrap();
        let nova = editar(&c, &r, "m1", &a.id, "GRAFANA Cloud").unwrap();
        assert_eq!(nova.palavra, "GRAFANA Cloud");
        assert_eq!(
            editar(&c, &r, "m1", &a.id, "dokploy").unwrap_err().codigo,
            "vocabulario_duplicada"
        );
        // mudar só a caixa da própria palavra é permitido
        editar(&c, &r, "m1", &a.id, "grafana cloud").unwrap();
        assert_eq!(
            editar(&c, &r, "m1", "inexistente", "x").unwrap_err().codigo,
            "vocabulario_nao_encontrada"
        );
    }

    #[test]
    fn dica_vazia_sem_palavras_e_junta_as_palavras_com_virgula() {
        let (c, r) = base();
        assert_eq!(dica(&c).unwrap(), None);
        add(&c, &r, "Going2").unwrap();
        add(&c, &r, "Kubernetes").unwrap();
        assert_eq!(dica(&c).unwrap().as_deref(), Some("Kubernetes, Going2"));
    }

    #[test]
    fn dica_respeita_o_orcamento_priorizando_as_mais_recentes() {
        let (c, r) = base();
        for i in 0..200 {
            add(&c, &r, &format!("termo-numero-{i:03}")).unwrap();
        }
        let d = dica(&c).unwrap().unwrap();
        assert!(d.chars().count() <= LIMITE_DICA_CARACTERES);
        assert!(
            d.starts_with("termo-numero-199"),
            "a mais recente entra primeiro"
        );
        assert!(
            !d.contains("termo-numero-000"),
            "as mais antigas ficam de fora"
        );
        let l = listar(&c, None).unwrap();
        assert_eq!(l.na_dica as usize, d.split(SEPARADOR_DICA).count());
        assert!(l.na_dica < l.total);
    }
}

//! Substituição de palavras (F-08): pares "original → substituto" aplicados ao texto transcrito,
//! antes da limpeza e do refinamento. A lista é sincronizável (escrita só por
//! `configuracao::gravar()`). O texto das substituições nunca vai para o log.

use crate::configuracao::{self, Tabela};
use crate::nucleo::{ErroApp, relogio::Relogio};
use rusqlite::{Connection, OptionalExtension, types::Value};
use serde::Serialize;
use specta::Type;

/// Tamanho máximo do original e do substituto, em caracteres.
pub const LIMITE_TAMANHO: usize = 200;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct Substituicao {
    pub id: String,
    pub original: String,
    pub substituto: String,
    pub ativa: bool,
}

/// Regra pronta para aplicar: só as ativas, já normalizadas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Regra {
    original: Vec<char>,
    substituto: String,
}

impl Regra {
    pub fn nova(original: &str, substituto: &str) -> Self {
        Self {
            original: normalizar(original).chars().collect(),
            substituto: substituto.to_string(),
        }
    }
}

/// Forma comparável: espaços sobrando colapsados, sem diferenciar maiúsculas.
pub fn normalizar(texto: &str) -> String {
    texto
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn limpar(texto: &str) -> String {
    texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn eh_palavra(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn mesmo_caractere(a: char, b: char) -> bool {
    a == b || a.to_lowercase().eq(b.to_lowercase())
}

/// Tamanho (em caracteres) do trecho de `texto` a partir de `i` que casa com o original da
/// regra, ou `None`. Um espaço do original casa com qualquer sequência de espaços do texto.
fn casar(regra: &Regra, texto: &[char], i: usize) -> Option<usize> {
    let mut t = i;
    for &o in &regra.original {
        if o == ' ' {
            if !texto.get(t).is_some_and(|c| c.is_whitespace()) {
                return None;
            }
            while texto.get(t).is_some_and(|c| c.is_whitespace()) {
                t += 1;
            }
        } else {
            if !texto.get(t).is_some_and(|&c| mesmo_caractere(c, o)) {
                return None;
            }
            t += 1;
        }
    }
    // Palavra inteira: se a borda do original é letra/dígito, o texto não pode continuar a palavra.
    let antes_ok = !regra.original.first().is_some_and(|&c| eh_palavra(c))
        || i == 0
        || !eh_palavra(texto[i - 1]);
    let depois_ok = !regra.original.last().is_some_and(|&c| eh_palavra(c))
        || t >= texto.len()
        || !eh_palavra(texto[t]);
    (antes_ok && depois_ok).then_some(t - i)
}

/// Aplica as regras ao texto (RF-08.2): palavras ou expressões inteiras, sem diferenciar
/// maiúsculas. Em uma passada só, vale a regra de original mais longo em cada posição; o
/// substituto nunca é reprocessado, então uma regra não dispara sobre o resultado de outra.
pub fn aplicar(regras: &[Regra], texto: &str) -> String {
    let mut ordenadas: Vec<&Regra> = regras.iter().filter(|r| !r.original.is_empty()).collect();
    ordenadas.sort_by_key(|r| std::cmp::Reverse(r.original.len()));
    let chars: Vec<char> = texto.chars().collect();
    let mut saida = String::with_capacity(texto.len());
    let mut i = 0;
    while i < chars.len() {
        let achada = ordenadas
            .iter()
            .find_map(|r| casar(r, &chars, i).map(|n| (r, n)));
        match achada {
            Some((regra, n)) if n > 0 => {
                saida.push_str(&regra.substituto);
                i += n;
            }
            _ => {
                saida.push(chars[i]);
                i += 1;
            }
        }
    }
    saida
}

fn validar(original: &str, substituto: &str) -> Result<(String, String, String), ErroApp> {
    let original = limpar(original);
    if original.is_empty() {
        return Err(ErroApp::novo(
            "substituicao_original_vazio",
            "Digite o que a transcrição costuma dizer.",
        )
        .com_campo("original"));
    }
    let substituto = substituto.trim().to_string();
    if substituto.is_empty() {
        return Err(ErroApp::novo(
            "substituicao_substituto_vazio",
            "Digite o texto que deve ser escrito no lugar.",
        )
        .com_campo("substituto"));
    }
    if original.chars().count() > LIMITE_TAMANHO {
        return Err(ErroApp::novo(
            "substituicao_texto_longo",
            "Use até 200 caracteres no original.",
        )
        .com_campo("original"));
    }
    if substituto.chars().count() > LIMITE_TAMANHO {
        return Err(ErroApp::novo(
            "substituicao_texto_longo",
            "Use até 200 caracteres no substituto.",
        )
        .com_campo("substituto"));
    }
    let normalizado = normalizar(&original);
    Ok((original, substituto, normalizado))
}

fn id_do_normalizado(conexao: &Connection, normalizado: &str) -> Result<Option<String>, ErroApp> {
    Ok(conexao
        .query_row(
            "SELECT id FROM substituicao WHERE original_normalizado = ?1 AND excluido_em IS NULL",
            [normalizado],
            |l| l.get(0),
        )
        .optional()?)
}

fn erro_duplicada(original: &str) -> ErroApp {
    ErroApp::novo(
        "substituicao_duplicada",
        &format!("Já existe uma substituição para “{original}”."),
    )
    .com_campo("original")
}

fn erro_nao_encontrada() -> ErroApp {
    ErroApp::novo(
        "substituicao_nao_encontrada",
        "Essa substituição não existe mais.",
    )
}

fn do_banco(l: &rusqlite::Row<'_>) -> rusqlite::Result<Substituicao> {
    Ok(Substituicao {
        id: l.get(0)?,
        original: l.get(1)?,
        substituto: l.get(2)?,
        ativa: l.get::<_, i64>(3)? != 0,
    })
}

fn buscar(conexao: &Connection, id: &str) -> Result<Option<Substituicao>, ErroApp> {
    Ok(conexao
        .query_row(
            "SELECT id, original, substituto, ativa FROM substituicao
             WHERE id = ?1 AND excluido_em IS NULL",
            [id],
            do_banco,
        )
        .optional()?)
}

/// Todas as substituições, as mais recentes primeiro (UUID v7 ordena por criação).
pub fn listar(conexao: &Connection) -> Result<Vec<Substituicao>, ErroApp> {
    let mut q = conexao.prepare(
        "SELECT id, original, substituto, ativa FROM substituicao
         WHERE excluido_em IS NULL ORDER BY id DESC",
    )?;
    let linhas = q.query_map([], do_banco)?.collect::<Result<Vec<_>, _>>()?;
    Ok(linhas)
}

fn gravar_linha(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    s: &Substituicao,
    normalizado: &str,
) -> Result<(), ErroApp> {
    configuracao::gravar(
        conexao,
        relogio,
        maquina,
        Tabela::Substituicao,
        &s.id,
        &[
            ("original", Value::from(s.original.clone())),
            ("original_normalizado", Value::from(normalizado.to_string())),
            ("substituto", Value::from(s.substituto.clone())),
            ("ativa", Value::from(i64::from(s.ativa))),
        ],
    )?;
    Ok(())
}

pub fn adicionar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    original: &str,
    substituto: &str,
) -> Result<Substituicao, ErroApp> {
    let (original, substituto, normalizado) = validar(original, substituto)?;
    if id_do_normalizado(conexao, &normalizado)?.is_some() {
        return Err(erro_duplicada(&original));
    }
    let nova = Substituicao {
        id: uuid::Uuid::now_v7().to_string(),
        original,
        substituto,
        ativa: true,
    };
    gravar_linha(conexao, relogio, maquina, &nova, &normalizado)?;
    Ok(nova)
}

pub fn editar(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
    original: &str,
    substituto: &str,
) -> Result<Substituicao, ErroApp> {
    let (original, substituto, normalizado) = validar(original, substituto)?;
    let atual = buscar(conexao, id)?.ok_or_else(erro_nao_encontrada)?;
    if id_do_normalizado(conexao, &normalizado)?.is_some_and(|outro| outro != id) {
        return Err(erro_duplicada(&original));
    }
    let nova = Substituicao {
        id: id.into(),
        original,
        substituto,
        ativa: atual.ativa,
    };
    gravar_linha(conexao, relogio, maquina, &nova, &normalizado)?;
    Ok(nova)
}

pub fn definir_ativa(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
    ativa: bool,
) -> Result<Substituicao, ErroApp> {
    let mut s = buscar(conexao, id)?.ok_or_else(erro_nao_encontrada)?;
    s.ativa = ativa;
    gravar_linha(conexao, relogio, maquina, &s, &normalizar(&s.original))?;
    Ok(s)
}

pub fn remover(
    conexao: &Connection,
    relogio: &Relogio,
    maquina: &str,
    id: &str,
) -> Result<(), ErroApp> {
    configuracao::excluir(conexao, relogio, maquina, Tabela::Substituicao, id)?;
    Ok(())
}

/// Regras ativas, para o ditado (RF-08.1: desativada não é aplicada).
pub fn regras_ativas(conexao: &Connection) -> Result<Vec<Regra>, ErroApp> {
    Ok(listar(conexao)?
        .into_iter()
        .filter(|s| s.ativa)
        .map(|s| Regra::nova(&s.original, &s.substituto))
        .collect())
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

    fn add(c: &Connection, r: &Relogio, o: &str, s: &str) -> Result<Substituicao, ErroApp> {
        adicionar(c, r, "m1", o, s)
    }

    fn regras(pares: &[(&str, &str)]) -> Vec<Regra> {
        pares.iter().map(|(o, s)| Regra::nova(o, s)).collect()
    }

    #[test]
    fn ignora_maiusculas_e_so_troca_palavra_inteira() {
        let r = regras(&[("vois desk", "Voice Desk")]);
        assert_eq!(aplicar(&r, "o Vois Desk"), "o Voice Desk");
        assert_eq!(aplicar(&r, "o voisdesking"), "o voisdesking");
        assert_eq!(aplicar(&r, "o vois desking"), "o vois desking");
        assert_eq!(aplicar(&r, "avois desk"), "avois desk");
    }

    #[test]
    fn manda_um_pix_vira_manda_um_pix_maiusculo() {
        let r = regras(&[("pix", "PIX")]);
        assert_eq!(aplicar(&r, "manda um pix"), "manda um PIX");
        assert_eq!(aplicar(&r, "pix, pix. Pix!"), "PIX, PIX. PIX!");
        assert_eq!(aplicar(&r, "pixel"), "pixel");
    }

    #[test]
    fn expressao_casa_com_espacos_diferentes_e_acentos() {
        let r = regras(&[("meu e-mail", "a@b.com"), ("ação", "AÇÃO")]);
        assert_eq!(
            aplicar(&r, "mande   meu\te-mail agora"),
            "mande   a@b.com agora"
        );
        assert_eq!(aplicar(&r, "Ação"), "AÇÃO");
        assert_eq!(aplicar(&r, "reação"), "reação");
    }

    #[test]
    fn substituto_e_literal_e_nao_e_reprocessado() {
        let r = regras(&[("a", "b"), ("b", "c"), ("custo", "$1 \\n (x)")]);
        assert_eq!(aplicar(&r, "a b"), "b c");
        assert_eq!(aplicar(&r, "custo"), "$1 \\n (x)");
    }

    #[test]
    fn original_mais_longo_tem_prioridade_e_texto_vazio_passa() {
        let r = regras(&[("voice", "V"), ("voice desk", "Voice Desk")]);
        assert_eq!(aplicar(&r, "voice desk e voice"), "Voice Desk e V");
        assert_eq!(aplicar(&r, ""), "");
        assert_eq!(aplicar(&[], "texto"), "texto");
    }

    #[test]
    fn original_com_simbolo_na_borda_casa_sem_exigir_limite_de_palavra() {
        let r = regras(&[("c++", "C plus plus")]);
        assert_eq!(
            aplicar(&r, "gosto de c++, mas"),
            "gosto de C plus plus, mas"
        );
    }

    #[test]
    fn criar_aparece_na_lista_mais_recente_primeiro() {
        let (c, r) = base();
        add(&c, &r, "pix", "PIX").unwrap();
        add(&c, &r, "gou tu", "Going2").unwrap();
        let l = listar(&c).unwrap();
        assert_eq!(l.len(), 2);
        assert_eq!(l[0].original, "gou tu");
        assert!(l[0].ativa);
    }

    #[test]
    fn original_vazio_ou_so_espacos_e_recusado_com_aviso() {
        let (c, r) = base();
        let e = add(&c, &r, "   ", "x").unwrap_err();
        assert_eq!(e.codigo, "substituicao_original_vazio");
        assert_eq!(e.campo.as_deref(), Some("original"));
        let e = add(&c, &r, "x", " ").unwrap_err();
        assert_eq!(e.codigo, "substituicao_substituto_vazio");
        let longo = "a".repeat(LIMITE_TAMANHO + 1);
        assert_eq!(
            add(&c, &r, &longo, "x").unwrap_err().codigo,
            "substituicao_texto_longo"
        );
        assert_eq!(
            add(&c, &r, "x", &longo).unwrap_err().codigo,
            "substituicao_texto_longo"
        );
        assert!(listar(&c).unwrap().is_empty());
    }

    #[test]
    fn duplicada_sem_diferenciar_maiusculas_e_recusada() {
        let (c, r) = base();
        add(&c, &r, "Pix", "PIX").unwrap();
        let e = add(&c, &r, "  pIX ", "outro").unwrap_err();
        assert_eq!(e.codigo, "substituicao_duplicada");
        assert_eq!(e.campo.as_deref(), Some("original"));
    }

    #[test]
    fn editar_troca_os_campos_mantem_o_estado_e_barra_duplicata() {
        let (c, r) = base();
        let a = add(&c, &r, "pix", "PIX").unwrap();
        add(&c, &r, "sla", "SLA").unwrap();
        definir_ativa(&c, &r, "m1", &a.id, false).unwrap();
        let nova = editar(&c, &r, "m1", &a.id, "PIX", "Pix").unwrap();
        assert_eq!(nova.substituto, "Pix");
        assert!(!nova.ativa, "editar não reativa");
        assert_eq!(
            editar(&c, &r, "m1", &a.id, "SLA", "x").unwrap_err().codigo,
            "substituicao_duplicada"
        );
        assert_eq!(
            editar(&c, &r, "m1", "nao-existe", "x", "y")
                .unwrap_err()
                .codigo,
            "substituicao_nao_encontrada"
        );
    }

    #[test]
    fn desativada_nao_e_aplicada_e_reativada_volta() {
        let (c, r) = base();
        let s = add(&c, &r, "pix", "PIX").unwrap();
        assert_eq!(aplicar(&regras_ativas(&c).unwrap(), "um pix"), "um PIX");
        definir_ativa(&c, &r, "m1", &s.id, false).unwrap();
        assert!(regras_ativas(&c).unwrap().is_empty());
        assert_eq!(aplicar(&regras_ativas(&c).unwrap(), "um pix"), "um pix");
        assert!(!listar(&c).unwrap()[0].ativa);
        definir_ativa(&c, &r, "m1", &s.id, true).unwrap();
        assert_eq!(aplicar(&regras_ativas(&c).unwrap(), "um pix"), "um PIX");
    }

    #[test]
    fn remover_libera_o_original_para_nova_inclusao() {
        let (c, r) = base();
        let s = add(&c, &r, "pix", "PIX").unwrap();
        remover(&c, &r, "m1", &s.id).unwrap();
        assert!(listar(&c).unwrap().is_empty());
        add(&c, &r, "pix", "Pix").unwrap();
        assert_eq!(listar(&c).unwrap().len(), 1);
    }
}

//! Regras das chaves: validação do formato, guarda no cofre, estado para a tela e leitura pelo
//! núcleo. O resultado do último teste vai para `preferencia_local` (tabela local, que não
//! sincroniza nem exporta) e nunca contém a chave.

use super::cofre::Cofre;
use super::{EstadoChave, Provedor, ResultadoTeste, Validade};
use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension, params};

const TAMANHO_MIN: usize = 8;
const TAMANHO_MAX: usize = 256;

/// Aceita só ASCII visível (a chave vai num cabeçalho HTTP), sem espaços no meio.
pub fn validar_formato(bruta: &str) -> Result<String, ErroApp> {
    let chave = bruta.trim();
    if chave.is_empty() {
        return Err(
            ErroApp::novo("chave_vazia", "Cole a chave de API do provedor.").com_campo("chave"),
        );
    }
    let n = chave.chars().count();
    if !(TAMANHO_MIN..=TAMANHO_MAX).contains(&n) || !chave.chars().all(|c| c.is_ascii_graphic()) {
        return Err(ErroApp::novo(
            "chave_formato",
            "Essa chave não parece completa. Confira se copiou inteira, sem espaços.",
        )
        .com_campo("chave"));
    }
    Ok(chave.to_string())
}

/// `••••abcd`: só os 4 últimos caracteres (RF-04.3).
pub fn mascara(chave: &str) -> String {
    let fim: Vec<char> = chave.chars().rev().take(4).collect();
    format!("••••{}", fim.into_iter().rev().collect::<String>())
}

fn chave_validade(provedor: Provedor) -> String {
    format!("chave_validade_{}", provedor.conta())
}

fn gravar_validade(
    conexao: &Connection,
    provedor: Provedor,
    teste: &ResultadoTeste,
) -> Result<(), ErroApp> {
    let valor = serde_json::to_string(teste).map_err(ErroApp::interno)?;
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT(chave) DO UPDATE SET valor = excluded.valor",
        params![chave_validade(provedor), valor],
    )?;
    Ok(())
}

fn ler_validade(
    conexao: &Connection,
    provedor: Provedor,
) -> Result<Option<ResultadoTeste>, ErroApp> {
    let valor: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = ?1",
            [chave_validade(provedor)],
            |l| l.get(0),
        )
        .optional()?;
    Ok(valor.and_then(|v| serde_json::from_str(&v).ok()))
}

pub fn estado(
    cofre: &dyn Cofre,
    conexao: &Connection,
    provedor: Provedor,
) -> Result<EstadoChave, ErroApp> {
    let Some(chave) = cofre.ler(provedor)? else {
        return Ok(EstadoChave {
            provedor,
            salva: false,
            mascara: None,
            validade: None,
            motivo: None,
        });
    };
    let teste = ler_validade(conexao, provedor)?;
    Ok(EstadoChave {
        provedor,
        salva: true,
        mascara: Some(mascara(&chave)),
        validade: Some(teste.as_ref().map_or(Validade::NaoTestada, |t| t.validade)),
        motivo: teste.and_then(|t| t.motivo),
    })
}

/// Guarda a chave no cofre (válida ou não: a tela deixa trocar) e registra o teste.
pub fn registrar(
    cofre: &dyn Cofre,
    conexao: &Connection,
    provedor: Provedor,
    chave: &str,
    teste: &ResultadoTeste,
) -> Result<EstadoChave, ErroApp> {
    cofre.gravar(provedor, chave)?;
    gravar_validade(conexao, provedor, teste)?;
    estado(cofre, conexao, provedor)
}

pub fn registrar_teste(
    cofre: &dyn Cofre,
    conexao: &Connection,
    provedor: Provedor,
    teste: &ResultadoTeste,
) -> Result<EstadoChave, ErroApp> {
    gravar_validade(conexao, provedor, teste)?;
    estado(cofre, conexao, provedor)
}

pub fn apagar(
    cofre: &dyn Cofre,
    conexao: &Connection,
    provedor: Provedor,
) -> Result<EstadoChave, ErroApp> {
    cofre.apagar(provedor)?;
    conexao.execute(
        "DELETE FROM preferencia_local WHERE chave = ?1",
        [chave_validade(provedor)],
    )?;
    estado(cofre, conexao, provedor)
}

/// Leitura pelo núcleo na hora da chamada ao provedor. A chave não volta para a interface.
/// Sem chave, o erro traz o aviso que o ditado mostra (RF-04.1).
pub fn obter(cofre: &dyn Cofre, provedor: Provedor) -> Result<String, ErroApp> {
    cofre.ler(provedor)?.ok_or_else(|| {
        ErroApp::novo(
            "chave_ausente",
            &format!(
                "Cadastre a chave do {} em Configurações › Provedores e chaves.",
                provedor.nome()
            ),
        )
    })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::chaves::cofre::CofreMemoria;
    use crate::nucleo::banco::{MIGRACOES, migrar};

    fn banco() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        migrar(&mut c, MIGRACOES).unwrap();
        c
    }

    fn valida() -> ResultadoTeste {
        ResultadoTeste {
            validade: Validade::Valida,
            motivo: None,
        }
    }

    #[test]
    fn mascara_mostra_so_os_quatro_ultimos() {
        assert_eq!(mascara("gsk_1234567890abcd"), "••••abcd");
    }

    #[test]
    fn formato_recusa_vazia_curta_com_espaco_e_nao_ascii() {
        assert_eq!(validar_formato("   ").unwrap_err().codigo, "chave_vazia");
        assert_eq!(
            validar_formato("curta").unwrap_err().codigo,
            "chave_formato"
        );
        assert_eq!(
            validar_formato("gsk_aaaa bbbb").unwrap_err().codigo,
            "chave_formato"
        );
        assert_eq!(
            validar_formato("gsk_ação_aaaa").unwrap_err().codigo,
            "chave_formato"
        );
        assert_eq!(
            validar_formato("gsk_aaaa\r\nX: y").unwrap_err().codigo,
            "chave_formato"
        );
        let longa = "a".repeat(257);
        assert_eq!(validar_formato(&longa).unwrap_err().codigo, "chave_formato");
        assert_eq!(
            validar_formato("  gsk_valida_1234  ").unwrap(),
            "gsk_valida_1234"
        );
        assert_eq!(
            validar_formato("").unwrap_err().campo.as_deref(),
            Some("chave")
        );
    }

    #[test]
    fn sem_chave_o_estado_diz_que_nao_ha() {
        let e = estado(&CofreMemoria::default(), &banco(), Provedor::Groq).unwrap();
        assert!(!e.salva);
        assert_eq!(e.mascara, None);
        assert_eq!(e.validade, None);
    }

    #[test]
    fn salvar_guarda_no_cofre_e_o_estado_so_mostra_a_mascara() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        let e = registrar(&cofre, &db, Provedor::Groq, "gsk_segredo_abcd", &valida()).unwrap();
        assert!(e.salva);
        assert_eq!(e.mascara.as_deref(), Some("••••abcd"));
        assert_eq!(e.validade, Some(Validade::Valida));
        assert!(!format!("{e:?}").contains("segredo"));
        assert_eq!(
            cofre.ler(Provedor::Groq).unwrap().as_deref(),
            Some("gsk_segredo_abcd")
        );
    }

    #[test]
    fn chave_nao_vai_para_nenhuma_tabela_do_banco() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        let teste = ResultadoTeste {
            validade: Validade::Invalida,
            motivo: Some("Invalid API Key".into()),
        };
        registrar(&cofre, &db, Provedor::Groq, "gsk_segredo_abcd", &teste).unwrap();
        let tabelas: Vec<String> = db
            .prepare(
                "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
            )
            .unwrap()
            .query_map([], |l| l.get(0))
            .unwrap()
            .map(Result::unwrap)
            .collect();
        for t in tabelas {
            let linhas: Vec<String> = {
                let mut s = db.prepare(&format!("SELECT * FROM \"{t}\"")).unwrap();
                let n = s.column_count();
                s.query_map([], |l| {
                    Ok((0..n)
                        .map(|i| format!("{:?}", l.get_ref(i).unwrap()))
                        .collect::<Vec<_>>()
                        .join(","))
                })
                .unwrap()
                .map(Result::unwrap)
                .collect()
            };
            assert!(
                linhas.iter().all(|l| !l.contains("segredo")),
                "chave encontrada na tabela {t}"
            );
        }
    }

    #[test]
    fn chave_invalida_fica_guardada_com_o_motivo() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        let teste = ResultadoTeste {
            validade: Validade::Invalida,
            motivo: Some("Invalid API Key".into()),
        };
        let e = registrar(&cofre, &db, Provedor::Groq, "gsk_inventada_0000", &teste).unwrap();
        assert_eq!(e.validade, Some(Validade::Invalida));
        assert_eq!(e.motivo.as_deref(), Some("Invalid API Key"));
    }

    #[test]
    fn trocar_substitui_a_chave_e_o_resultado_do_teste() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        let ruim = ResultadoTeste {
            validade: Validade::Invalida,
            motivo: Some("x".into()),
        };
        registrar(&cofre, &db, Provedor::Groq, "gsk_antiga_1111", &ruim).unwrap();
        let e = registrar(&cofre, &db, Provedor::Groq, "gsk_nova_2222", &valida()).unwrap();
        assert_eq!(e.mascara.as_deref(), Some("••••2222"));
        assert_eq!(e.validade, Some(Validade::Valida));
        assert_eq!(e.motivo, None);
    }

    #[test]
    fn apagar_remove_do_cofre_e_o_ditado_passa_a_avisar() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        registrar(&cofre, &db, Provedor::Groq, "gsk_segredo_abcd", &valida()).unwrap();
        let e = apagar(&cofre, &db, Provedor::Groq).unwrap();
        assert!(!e.salva);
        let erro = obter(&cofre, Provedor::Groq).unwrap_err();
        assert_eq!(erro.codigo, "chave_ausente");
        assert_eq!(
            erro.mensagem,
            "Cadastre a chave do Groq em Configurações › Provedores e chaves."
        );
        // Apagar de novo não dá erro.
        apagar(&cofre, &db, Provedor::Groq).unwrap();
    }

    #[test]
    fn obter_devolve_a_chave_para_o_nucleo() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        registrar(&cofre, &db, Provedor::Groq, "gsk_segredo_abcd", &valida()).unwrap();
        assert_eq!(obter(&cofre, Provedor::Groq).unwrap(), "gsk_segredo_abcd");
    }

    #[test]
    fn retestar_atualiza_so_a_validade() {
        let (cofre, db) = (CofreMemoria::default(), banco());
        let sem_rede = ResultadoTeste {
            validade: Validade::NaoTestada,
            motivo: Some("sem rede".into()),
        };
        registrar(&cofre, &db, Provedor::Groq, "gsk_segredo_abcd", &sem_rede).unwrap();
        let e = registrar_teste(&cofre, &db, Provedor::Groq, &valida()).unwrap();
        assert_eq!(e.validade, Some(Validade::Valida));
        assert_eq!(e.mascara.as_deref(), Some("••••abcd"));
    }
}

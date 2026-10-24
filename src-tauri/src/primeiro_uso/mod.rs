//! Primeiro uso guiado (F-06): guarda, só nesta máquina, se o passo a passo terminou e quais
//! passos foram pulados. Tudo em `preferencia_local`, que nunca é sincronizada (RF-06 vale por máquina).

use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension};
use serde::Serialize;
use specta::Type;

const CHAVE_CONCLUIDO: &str = "primeiro_uso.concluido";
const CHAVE_PULADOS: &str = "primeiro_uso.pulados";

/// Passos que podem ser pulados e retomados depois pelas configurações (RF-06.2).
pub const PASSOS: [&str; 4] = ["permissoes", "atalho", "provedor", "sincronizacao"];

#[derive(Debug, Clone, PartialEq, Serialize, Type)]
pub struct EstadoPrimeiroUso {
    pub concluido: bool,
    pub pulados: Vec<String>,
}

fn ler_valor(c: &Connection, chave: &str) -> Result<Option<String>, ErroApp> {
    Ok(c.query_row(
        "SELECT valor FROM preferencia_local WHERE chave = ?1",
        [chave],
        |l| l.get(0),
    )
    .optional()?)
}

fn gravar_valor(c: &Connection, chave: &str, valor: &str) -> Result<(), ErroApp> {
    c.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        rusqlite::params![chave, valor],
    )?;
    Ok(())
}

fn validar(passo: &str) -> Result<(), ErroApp> {
    if PASSOS.contains(&passo) {
        Ok(())
    } else {
        Err(ErroApp::novo("passo_desconhecido", "Esse passo não existe.").com_campo("passo"))
    }
}

pub fn estado(c: &Connection) -> Result<EstadoPrimeiroUso, ErroApp> {
    let concluido = ler_valor(c, CHAVE_CONCLUIDO)?.as_deref() == Some("true");
    let guardados: Vec<String> = ler_valor(c, CHAVE_PULADOS)?
        .and_then(|v| serde_json::from_str(&v).ok())
        .unwrap_or_default();
    // Na ordem do passo a passo, ignorando lixo que não seja um passo conhecido.
    let pulados = PASSOS
        .iter()
        .filter(|p| guardados.iter().any(|g| g == *p))
        .map(|p| p.to_string())
        .collect();
    Ok(EstadoPrimeiroUso { concluido, pulados })
}

fn gravar_pulados(c: &Connection, pulados: &[String]) -> Result<(), ErroApp> {
    let json = serde_json::to_string(pulados).map_err(ErroApp::interno)?;
    gravar_valor(c, CHAVE_PULADOS, &json)
}

/// Marca o passo como pulado (repetir não duplica).
pub fn pular(c: &Connection, passo: &str) -> Result<EstadoPrimeiroUso, ErroApp> {
    validar(passo)?;
    let mut atual = estado(c)?;
    if !atual.pulados.iter().any(|p| p == passo) {
        atual.pulados.push(passo.to_string());
        gravar_pulados(c, &atual.pulados)?;
    }
    estado(c)
}

/// Tira a marca de pulado quando o usuário faz o passo (aqui ou nas configurações).
pub fn cumprir(c: &Connection, passo: &str) -> Result<EstadoPrimeiroUso, ErroApp> {
    validar(passo)?;
    let mut atual = estado(c)?;
    let antes = atual.pulados.len();
    atual.pulados.retain(|p| p != passo);
    if atual.pulados.len() != antes {
        gravar_pulados(c, &atual.pulados)?;
    }
    estado(c)
}

/// Encerra o primeiro uso: não aparece de novo ao reabrir o app (critério de aceite).
pub fn concluir(c: &Connection) -> Result<EstadoPrimeiroUso, ErroApp> {
    gravar_valor(c, CHAVE_CONCLUIDO, "true")?;
    estado(c)
}

type Banco = std::sync::Mutex<Connection>;

fn travar(b: &Banco) -> Result<std::sync::MutexGuard<'_, Connection>, ErroApp> {
    b.lock().map_err(|_| ErroApp::interno("trava do banco"))
}

#[tauri::command]
#[specta::specta]
pub fn primeiro_uso_estado(banco: tauri::State<'_, Banco>) -> Result<EstadoPrimeiroUso, ErroApp> {
    estado(&*travar(&banco)?)
}

#[tauri::command]
#[specta::specta]
pub fn primeiro_uso_pular(
    banco: tauri::State<'_, Banco>,
    passo: String,
) -> Result<EstadoPrimeiroUso, ErroApp> {
    pular(&*travar(&banco)?, &passo)
}

#[tauri::command]
#[specta::specta]
pub fn primeiro_uso_cumprir(
    banco: tauri::State<'_, Banco>,
    passo: String,
) -> Result<EstadoPrimeiroUso, ErroApp> {
    cumprir(&*travar(&banco)?, &passo)
}

#[tauri::command]
#[specta::specta]
pub fn primeiro_uso_concluir(banco: tauri::State<'_, Banco>) -> Result<EstadoPrimeiroUso, ErroApp> {
    concluir(&*travar(&banco)?)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco;

    fn conexao() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        banco::migrar(&mut c, banco::MIGRACOES).unwrap();
        c
    }

    #[test]
    fn maquina_nova_nao_concluiu_e_nao_pulou_nada() {
        let e = estado(&conexao()).unwrap();
        assert_eq!(
            e,
            EstadoPrimeiroUso {
                concluido: false,
                pulados: vec![]
            }
        );
    }

    #[test]
    fn concluir_vale_nas_proximas_aberturas() {
        let c = conexao();
        assert!(concluir(&c).unwrap().concluido);
        assert!(estado(&c).unwrap().concluido);
    }

    #[test]
    fn pular_a_chave_nao_impede_de_concluir() {
        let c = conexao();
        let e = pular(&c, "provedor").unwrap();
        assert_eq!(e.pulados, vec!["provedor"]);
        assert!(concluir(&c).unwrap().concluido);
        assert_eq!(estado(&c).unwrap().pulados, vec!["provedor"]);
    }

    #[test]
    fn pular_duas_vezes_nao_duplica_e_mantem_a_ordem_do_passo_a_passo() {
        let c = conexao();
        pular(&c, "sincronizacao").unwrap();
        pular(&c, "atalho").unwrap();
        let e = pular(&c, "atalho").unwrap();
        assert_eq!(e.pulados, vec!["atalho", "sincronizacao"]);
    }

    #[test]
    fn cumprir_tira_a_marca_de_pulado() {
        let c = conexao();
        pular(&c, "provedor").unwrap();
        assert!(cumprir(&c, "provedor").unwrap().pulados.is_empty());
    }

    #[test]
    fn passo_desconhecido_e_recusado_com_o_campo() {
        let c = conexao();
        let e = pular(&c, "ditado").unwrap_err();
        assert_eq!(e.codigo, "passo_desconhecido");
        assert_eq!(e.campo.as_deref(), Some("passo"));
        assert!(cumprir(&c, "x").is_err());
    }

    #[test]
    fn valor_inesperado_nos_pulados_e_ignorado() {
        let c = conexao();
        gravar_valor(&c, CHAVE_PULADOS, r#"{"nao":"lista"}"#).unwrap();
        assert!(estado(&c).unwrap().pulados.is_empty());
        gravar_valor(&c, CHAVE_PULADOS, r#"["atalho","lixo"]"#).unwrap();
        assert_eq!(estado(&c).unwrap().pulados, vec!["atalho"]);
    }
}

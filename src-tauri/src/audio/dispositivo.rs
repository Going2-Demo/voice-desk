//! Escolha do microfone (F-23): a preferência é local (`preferencia_local`, não sincroniza) e
//! guarda o nome do dispositivo; sem registro vale o padrão do sistema.

use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;

pub const CHAVE: &str = "audio.microfone";
const TAMANHO_MAXIMO: usize = 200;

/// O que o ditado vai usar, dado o que foi escolhido e o que está conectado.
#[derive(Debug, Clone, PartialEq)]
pub enum Escolha {
    /// Padrão do sistema, por escolha do usuário.
    Padrao,
    /// O microfone escolhido está conectado.
    Preferido(String),
    /// O microfone escolhido não está conectado: usa o padrão e avisa (RF-23.2).
    Ausente(String),
}

pub fn resolver(preferido: Option<&str>, conectados: &[String]) -> Escolha {
    match preferido {
        None => Escolha::Padrao,
        Some(nome) if conectados.iter().any(|c| c == nome) => Escolha::Preferido(nome.into()),
        Some(nome) => Escolha::Ausente(nome.into()),
    }
}

pub fn aviso_ausente(nome: &str) -> String {
    format!("Usando o microfone padrão: “{nome}” desconectado")
}

/// Estado da tela de Configurações · Geral › Microfone.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EstadoMicrofone {
    /// Dispositivos de entrada conectados agora.
    pub dispositivos: Vec<String>,
    /// Nome escolhido; `None` = padrão do sistema.
    pub escolhido: Option<String>,
    /// O escolhido está conectado (sempre `true` no padrão do sistema).
    pub conectado: bool,
}

pub fn montar_estado(escolhido: Option<String>, dispositivos: Vec<String>) -> EstadoMicrofone {
    let conectado = !matches!(
        resolver(escolhido.as_deref(), &dispositivos),
        Escolha::Ausente(_)
    );
    EstadoMicrofone {
        dispositivos,
        escolhido,
        conectado,
    }
}

pub fn ler(conexao: &Connection) -> Result<Option<String>, ErroApp> {
    let json: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = ?1",
            [CHAVE],
            |l| l.get(0),
        )
        .optional()?;
    // `valor` é JSON; um valor que não seja texto vale como "padrão do sistema".
    Ok(json
        .and_then(|j| serde_json::from_str::<String>(&j).ok())
        .filter(|v| !v.is_empty()))
}

/// Guarda a escolha (`None` = padrão do sistema). Só aceita um dispositivo conectado agora.
pub fn gravar(
    conexao: &Connection,
    escolhido: Option<&str>,
    conectados: &[String],
) -> Result<(), ErroApp> {
    let Some(nome) = escolhido else {
        conexao.execute("DELETE FROM preferencia_local WHERE chave = ?1", [CHAVE])?;
        return Ok(());
    };
    if nome.is_empty() || nome.chars().count() > TAMANHO_MAXIMO {
        return Err(
            ErroApp::novo("microfone_invalido", "Escolha um microfone da lista.")
                .com_campo("microfone"),
        );
    }
    if !conectados.iter().any(|c| c == nome) {
        return Err(ErroApp::novo(
            "microfone_desconectado",
            "Este microfone não está conectado. Escolha um da lista.",
        )
        .com_campo("microfone"));
    }
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        rusqlite::params![
            CHAVE,
            serde_json::to_string(nome).map_err(ErroApp::interno)?
        ],
    )?;
    Ok(())
}

/// Nomes dos dispositivos de entrada conectados, sem repetição e em ordem alfabética.
pub fn listar_entradas() -> Vec<String> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let mut nomes: Vec<String> = cpal::default_host()
        .input_devices()
        .map(|ds| {
            ds.filter_map(|d| d.description().ok().map(|d| d.name().to_string()))
                .collect()
        })
        .unwrap_or_default();
    nomes.sort();
    nomes.dedup();
    nomes
}

pub fn estado(conexao: &Connection) -> Result<EstadoMicrofone, ErroApp> {
    Ok(montar_estado(ler(conexao)?, listar_entradas()))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco;

    fn base() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        banco::migrar(&mut c, banco::MIGRACOES).unwrap();
        c
    }

    fn lista(nomes: &[&str]) -> Vec<String> {
        nomes.iter().map(|n| n.to_string()).collect()
    }

    #[test]
    fn sem_escolha_vale_o_padrao_do_sistema() {
        assert_eq!(resolver(None, &lista(&["Headset"])), Escolha::Padrao);
    }

    #[test]
    fn escolhido_conectado_e_usado() {
        let c = lista(&["Interno", "Headset"]);
        assert_eq!(
            resolver(Some("Headset"), &c),
            Escolha::Preferido("Headset".into())
        );
    }

    #[test]
    fn escolhido_desconectado_cai_no_padrao_e_volta_ao_reconectar() {
        let sem = lista(&["Interno"]);
        assert_eq!(
            resolver(Some("Headset"), &sem),
            Escolha::Ausente("Headset".into())
        );
        let com = lista(&["Interno", "Headset"]);
        assert_eq!(
            resolver(Some("Headset"), &com),
            Escolha::Preferido("Headset".into())
        );
    }

    #[test]
    fn aviso_diz_qual_microfone_ficou_de_fora() {
        assert_eq!(
            aviso_ausente("Headset USB"),
            "Usando o microfone padrão: “Headset USB” desconectado"
        );
    }

    #[test]
    fn preferencia_persiste_e_volta_ao_padrao() {
        let c = base();
        assert_eq!(ler(&c).unwrap(), None);
        gravar(&c, Some("Headset"), &lista(&["Headset"])).unwrap();
        assert_eq!(ler(&c).unwrap().as_deref(), Some("Headset"));
        gravar(&c, None, &[]).unwrap();
        assert_eq!(ler(&c).unwrap(), None);
    }

    #[test]
    fn recusa_microfone_fora_da_lista() {
        let c = base();
        let e = gravar(&c, Some("Fantasma"), &lista(&["Headset"])).unwrap_err();
        assert_eq!(e.codigo, "microfone_desconectado");
        assert_eq!(e.campo.as_deref(), Some("microfone"));
        assert_eq!(ler(&c).unwrap(), None);
    }

    #[test]
    fn recusa_nome_vazio_ou_longo_demais() {
        let c = base();
        assert_eq!(
            gravar(&c, Some(""), &lista(&[""])).unwrap_err().codigo,
            "microfone_invalido"
        );
        let longo = "a".repeat(201);
        assert_eq!(
            gravar(&c, Some(&longo), std::slice::from_ref(&longo))
                .unwrap_err()
                .codigo,
            "microfone_invalido"
        );
    }

    #[test]
    fn estado_marca_o_escolhido_ausente() {
        let e = montar_estado(Some("Headset".into()), lista(&["Interno"]));
        assert!(!e.conectado);
        let e = montar_estado(Some("Headset".into()), lista(&["Interno", "Headset"]));
        assert!(e.conectado);
        let e = montar_estado(None, vec![]);
        assert!(e.conectado);
    }
}

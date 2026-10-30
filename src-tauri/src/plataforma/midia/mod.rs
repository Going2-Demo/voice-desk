//! Pausar a mídia durante o ditado (F-25). A pausa só vale se havia mídia tocando (RF-25.2) e só
//! se a preferência local está ligada (RF-25.1, padrão ligado). Mac: MediaRemote (API privada da
//! Apple, carregada em tempo de execução; se o sistema a restringir, nada acontece e o ditado
//! segue). Windows: controles de mídia do sistema (SMTC). Programa que não aceita o controle
//! não é afetado.

use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod windows;

pub const CHAVE: &str = "midia.pausar_no_ditado";

/// O que cada sistema sabe fazer com a mídia.
pub trait Controle: Send + Sync {
    /// Pausa o que estiver tocando e devolve a lista do que foi pausado por nós (vazia se nada
    /// tocava ou o sistema não aceitou o controle).
    fn pausar_tocando(&self) -> Vec<Pausada>;
    /// Retoma exatamente o que `pausar_tocando` pausou.
    fn retomar(&self, pausadas: &[Pausada]);
}

/// Identifica uma sessão de mídia pausada por nós (id do aplicativo; vazio no Mac).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pausada(pub String);

/// Controle do sistema atual.
pub fn do_sistema() -> Box<dyn Controle> {
    #[cfg(target_os = "macos")]
    return Box::new(macos::MediaRemote);
    #[cfg(not(target_os = "macos"))]
    return Box::new(windows::Smtc);
}

/// Guarda o que o ditado pausou, para só retomar isso (RF-25.2).
pub struct PausaDeMidia {
    controle: Box<dyn Controle>,
    pausadas: Mutex<Vec<Pausada>>,
}

impl PausaDeMidia {
    pub fn nova(controle: Box<dyn Controle>) -> Self {
        Self {
            controle,
            pausadas: Mutex::new(Vec::new()),
        }
    }

    /// Pausa a mídia que toca. Chamada de novo sem retomar, não pausa duas vezes.
    pub fn pausar(&self) {
        let Ok(mut guardadas) = self.pausadas.lock() else {
            return;
        };
        if guardadas.is_empty() {
            *guardadas = self.controle.pausar_tocando();
        }
    }

    /// Retoma só o que foi pausado antes; sem nada pausado, não faz nada.
    pub fn retomar(&self) {
        let Ok(mut guardadas) = self.pausadas.lock() else {
            return;
        };
        if !guardadas.is_empty() {
            self.controle.retomar(&guardadas);
            guardadas.clear();
        }
    }
}

/// Preferência local; sem registro, o padrão é ligado.
pub fn ler(conexao: &Connection) -> Result<bool, ErroApp> {
    let valor: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = ?1",
            [CHAVE],
            |l| l.get(0),
        )
        .optional()?;
    Ok(valor.as_deref().map(|v| v != "false").unwrap_or(true))
}

pub fn gravar(conexao: &Connection, ligado: bool) -> Result<bool, ErroApp> {
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        rusqlite::params![CHAVE, if ligado { "true" } else { "false" }],
    )?;
    Ok(ligado)
}

#[tauri::command]
#[specta::specta]
pub fn midia_pausar_obter(banco: tauri::State<Mutex<Connection>>) -> Result<bool, ErroApp> {
    ler(&*banco.lock().map_err(ErroApp::interno)?)
}

#[tauri::command]
#[specta::specta]
pub fn midia_pausar_definir(
    banco: tauri::State<Mutex<Connection>>,
    ligado: bool,
) -> Result<bool, ErroApp> {
    gravar(&*banco.lock().map_err(ErroApp::interno)?, ligado)
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::sync::Arc;

    #[derive(Default)]
    struct Falso {
        tocando: Mutex<Vec<String>>,
        chamadas: Mutex<Vec<String>>,
    }

    impl Controle for Arc<Falso> {
        fn pausar_tocando(&self) -> Vec<Pausada> {
            self.chamadas.lock().unwrap().push("pausar".into());
            std::mem::take(&mut *self.tocando.lock().unwrap())
                .into_iter()
                .map(Pausada)
                .collect()
        }
        fn retomar(&self, pausadas: &[Pausada]) {
            self.chamadas
                .lock()
                .unwrap()
                .push(format!("retomar:{}", pausadas.len()));
        }
    }

    fn pausa(tocando: &[&str]) -> (PausaDeMidia, Arc<Falso>) {
        let falso = Arc::new(Falso::default());
        *falso.tocando.lock().unwrap() = tocando.iter().map(|s| s.to_string()).collect();
        (PausaDeMidia::nova(Box::new(falso.clone())), falso)
    }

    #[test]
    fn retoma_o_que_estava_tocando() {
        let (p, f) = pausa(&["spotify"]);
        p.pausar();
        p.retomar();
        assert_eq!(*f.chamadas.lock().unwrap(), ["pausar", "retomar:1"]);
    }

    #[test]
    fn nao_retoma_o_que_ja_estava_pausado() {
        let (p, f) = pausa(&[]);
        p.pausar();
        p.retomar();
        assert_eq!(*f.chamadas.lock().unwrap(), ["pausar"]);
    }

    #[test]
    fn retomar_sem_pausar_nao_faz_nada() {
        let (p, f) = pausa(&["x"]);
        p.retomar();
        assert!(f.chamadas.lock().unwrap().is_empty());
    }

    #[test]
    fn retomar_so_uma_vez() {
        let (p, f) = pausa(&["x"]);
        p.pausar();
        p.retomar();
        p.retomar();
        assert_eq!(f.chamadas.lock().unwrap().len(), 2);
    }

    #[test]
    fn preferencia_vem_ligada_e_grava_desligada() {
        let c = Connection::open_in_memory().unwrap();
        c.execute(
            "CREATE TABLE preferencia_local (chave TEXT PRIMARY KEY, valor TEXT NOT NULL)",
            [],
        )
        .unwrap();
        assert!(ler(&c).unwrap());
        gravar(&c, false).unwrap();
        assert!(!ler(&c).unwrap());
        gravar(&c, true).unwrap();
        assert!(ler(&c).unwrap());
    }
}

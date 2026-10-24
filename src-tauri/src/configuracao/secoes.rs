//! Seções da configuração sincronizável. Cada área implementa a sua `Secao` na própria pasta; a
//! lista completa nasce aqui, com seções "pendentes", e a história da área troca a sua.

use crate::nucleo::ErroApp;
use serde_json::Value;

pub trait Secao: Send + Sync {
    fn nome(&self) -> &'static str;
    fn exportar(&self) -> Result<Value, ErroApp>;
    fn validar(&self, dados: &Value) -> Result<(), ErroApp>;
    fn importar(&self, dados: &Value) -> Result<(), ErroApp>;
    fn mesclar(&self, local: &Value, remoto: &Value) -> Result<Value, ErroApp>;
}

/// Ordem fixa: a ordem do arquivo de configuração não muda sem passar pelo Otávio.
pub const NOMES: &[&str] = &[
    "modos",
    "modo_programas",
    "palavras_gatilho",
    "vocabulario",
    "substituicoes",
    "palavras_preenchimento",
    "preferencias_sincronizadas",
];

struct Pendente(&'static str);

impl Secao for Pendente {
    fn nome(&self) -> &'static str {
        self.0
    }
    fn exportar(&self) -> Result<Value, ErroApp> {
        Ok(Value::Array(vec![]))
    }
    fn validar(&self, _: &Value) -> Result<(), ErroApp> {
        Ok(())
    }
    fn importar(&self, _: &Value) -> Result<(), ErroApp> {
        Ok(())
    }
    fn mesclar(&self, local: &Value, _: &Value) -> Result<Value, ErroApp> {
        Ok(local.clone())
    }
}

pub fn todas() -> Vec<Box<dyn Secao>> {
    NOMES
        .iter()
        .map(|n| Box::new(Pendente(n)) as Box<dyn Secao>)
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn lista_completa_na_ordem() {
        let nomes: Vec<_> = todas().iter().map(|s| s.nome()).collect();
        assert_eq!(nomes, NOMES);
    }

    #[test]
    fn secao_pendente_exporta_lista_vazia_e_mantem_o_local_na_mescla() {
        let s = &todas()[0];
        assert_eq!(s.exportar().unwrap(), serde_json::json!([]));
        let local = serde_json::json!([1]);
        assert_eq!(s.mesclar(&local, &serde_json::json!([2])).unwrap(), local);
    }
}

//! Cofre de chaves: Keychain no Mac, Gerenciador de Credenciais no Windows (crate `keyring`).
//! O trait existe para os testes usarem um cofre em memória.

use super::Provedor;
use crate::nucleo::ErroApp;
use std::collections::HashMap;
use std::sync::Mutex;

pub const SERVICO: &str = "voice-desk";

pub trait Cofre: Send + Sync {
    fn ler(&self, provedor: Provedor) -> Result<Option<String>, ErroApp>;
    fn gravar(&self, provedor: Provedor, chave: &str) -> Result<(), ErroApp>;
    /// Apagar uma chave que não existe não é erro.
    fn apagar(&self, provedor: Provedor) -> Result<(), ErroApp>;
}

fn erro_cofre(e: keyring::Error) -> ErroApp {
    // O detalhe do cofre não carrega a chave; mesmo assim a interface só recebe a mensagem fixa.
    tracing::error!(evento = "erro_interno", origem = "cofre", detalhe = %e);
    ErroApp::novo(
        "cofre_indisponivel",
        "Não consegui acessar o cofre de chaves do sistema. Desbloqueie o cofre e tente de novo.",
    )
}

pub struct CofreSistema {
    servico: &'static str,
}

impl CofreSistema {
    pub fn novo() -> Self {
        Self { servico: SERVICO }
    }

    fn entrada(&self, provedor: Provedor) -> Result<keyring::Entry, ErroApp> {
        keyring::Entry::new(self.servico, provedor.conta()).map_err(erro_cofre)
    }
}

impl Cofre for CofreSistema {
    fn ler(&self, provedor: Provedor) -> Result<Option<String>, ErroApp> {
        match self.entrada(provedor)?.get_password() {
            Ok(chave) => Ok(Some(chave)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(erro_cofre(e)),
        }
    }

    fn gravar(&self, provedor: Provedor, chave: &str) -> Result<(), ErroApp> {
        self.entrada(provedor)?
            .set_password(chave)
            .map_err(erro_cofre)
    }

    fn apagar(&self, provedor: Provedor) -> Result<(), ErroApp> {
        match self.entrada(provedor)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(erro_cofre(e)),
        }
    }
}

/// Cofre em memória, só para testes.
#[derive(Default)]
pub struct CofreMemoria(Mutex<HashMap<&'static str, String>>);

impl Cofre for CofreMemoria {
    fn ler(&self, provedor: Provedor) -> Result<Option<String>, ErroApp> {
        Ok(self.0.lock().unwrap().get(provedor.conta()).cloned())
    }

    fn gravar(&self, provedor: Provedor, chave: &str) -> Result<(), ErroApp> {
        self.0
            .lock()
            .unwrap()
            .insert(provedor.conta(), chave.to_string());
        Ok(())
    }

    fn apagar(&self, provedor: Provedor) -> Result<(), ErroApp> {
        self.0.lock().unwrap().remove(provedor.conta());
        Ok(())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn memoria_grava_le_e_apaga() {
        let c = CofreMemoria::default();
        assert_eq!(c.ler(Provedor::Groq).unwrap(), None);
        c.gravar(Provedor::Groq, "gsk_abc").unwrap();
        assert_eq!(c.ler(Provedor::Groq).unwrap().as_deref(), Some("gsk_abc"));
        c.apagar(Provedor::Groq).unwrap();
        c.apagar(Provedor::Groq).unwrap();
        assert_eq!(c.ler(Provedor::Groq).unwrap(), None);
    }

    /// Toca o cofre real da máquina; rode à mão: `cargo test cofre_do_sistema -- --ignored`.
    #[test]
    #[ignore]
    fn cofre_do_sistema_faz_o_ciclo_completo() {
        // Serviço próprio: não encosta na chave real do usuário.
        let c = CofreSistema {
            servico: "voice-desk-teste",
        };
        c.gravar(Provedor::Groq, "gsk_teste_local_1234").unwrap();
        assert_eq!(
            c.ler(Provedor::Groq).unwrap().as_deref(),
            Some("gsk_teste_local_1234")
        );
        c.apagar(Provedor::Groq).unwrap();
        assert_eq!(c.ler(Provedor::Groq).unwrap(), None);
    }
}

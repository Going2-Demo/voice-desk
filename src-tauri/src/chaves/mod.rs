//! Chaves de API dos provedores de nuvem (F-04). Ficam só no cofre do sistema (serviço
//! `voice-desk`, uma conta por provedor); nunca no banco, em log, na exportação nem na
//! sincronização. A interface só recebe a máscara `••••abcd`.

pub mod cofre;
pub mod groq;
pub mod servico;

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Provedor {
    Groq,
}

impl Provedor {
    /// Conta no cofre do sistema.
    pub fn conta(self) -> &'static str {
        match self {
            Provedor::Groq => "groq",
        }
    }

    pub fn nome(self) -> &'static str {
        match self {
            Provedor::Groq => "Groq",
        }
    }

    /// Página do provedor onde se obtém a chave (RF-04.4). Fixa no núcleo: a interface nunca
    /// escolhe a URL aberta no navegador.
    pub fn pagina_da_chave(self) -> &'static str {
        match self {
            Provedor::Groq => "https://console.groq.com/keys",
        }
    }

    pub fn todos() -> &'static [Provedor] {
        &[Provedor::Groq]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum Validade {
    /// O provedor aceitou a chave.
    Valida,
    /// O provedor recusou a chave.
    Invalida,
    /// Não deu para testar (sem rede, tempo esgotado, provedor fora do ar).
    NaoTestada,
}

/// Resultado do teste no provedor; `motivo` é o texto informado por ele (RF-04.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct ResultadoTeste {
    pub validade: Validade,
    pub motivo: Option<String>,
}

/// Estado de uma chave para a tela: nunca contém a chave inteira.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Type)]
pub struct EstadoChave {
    pub provedor: Provedor,
    pub salva: bool,
    /// `••••abcd` (RF-04.3); `None` sem chave.
    pub mascara: Option<String>,
    pub validade: Option<Validade>,
    pub motivo: Option<String>,
}

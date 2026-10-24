//! Transcrição (F-02): contrato `Transcritor` e o provedor de nuvem Groq. O provedor, o modelo e
//! o idioma vêm do modo ativo (`config`). Falhas viram `ErroApp` com mensagem pronta em português
//! (RF-02.3); o texto ditado e a chave nunca vão para o log.

pub mod config;
pub mod falha;
pub mod flac;
pub mod groq;

use crate::audio::captura::Gravacao;
use crate::chaves::{self, Provedor, cofre::Cofre};
use crate::nucleo::ErroApp;
use std::sync::Arc;

pub use config::{ConfigTranscricao, Idioma, ProvedorTranscricao};

/// Tempo máximo à espera do provedor (RF-02.3).
pub const LIMITE_RESPOSTA_S: u64 = 30;

/// O que o provedor recebe: áudio FLAC 16 kHz mono, idioma e dica de vocabulário opcional.
#[derive(Debug, Clone)]
pub struct Pedido {
    pub audio_flac: Vec<u8>,
    pub modelo: String,
    pub idioma: Idioma,
    pub dica: Option<String>,
}

pub trait Transcritor: Send + Sync {
    fn transcrever(&self, pedido: &Pedido) -> Result<String, ErroApp>;
}

/// Monta o transcritor do modo na hora do uso (lê a chave do cofre só então).
pub type Fabrica =
    dyn Fn(&ConfigTranscricao) -> Result<Box<dyn Transcritor>, ErroApp> + Send + Sync;

/// Fábrica compartilhada que atravessa o `Contexto` do ditado.
#[derive(Clone)]
pub struct FabricaCompartilhada(pub Arc<Fabrica>);

impl std::fmt::Debug for FabricaCompartilhada {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("FabricaCompartilhada")
    }
}

impl PartialEq for FabricaCompartilhada {
    fn eq(&self, outra: &Self) -> bool {
        Arc::ptr_eq(&self.0, &outra.0)
    }
}

/// Converte, envia e devolve o texto transcrito (sem espaços nas pontas).
pub fn transcrever(
    transcritor: &dyn Transcritor,
    config: &ConfigTranscricao,
    gravacao: &Gravacao,
    dica: Option<String>,
) -> Result<String, ErroApp> {
    let pedido = Pedido {
        audio_flac: flac::codificar(gravacao)?,
        modelo: config.modelo.clone(),
        idioma: config.idioma,
        dica,
    };
    let texto = transcritor.transcrever(&pedido)?;
    Ok(texto.trim().to_string())
}

pub fn erro_sem_conexao() -> ErroApp {
    ErroApp::novo(
        "sem_conexao",
        "Sem conexão com o provedor. Confira a internet e tente de novo.",
    )
}

pub fn erro_tempo_esgotado() -> ErroApp {
    ErroApp::novo(
        "tempo_esgotado",
        "O provedor não respondeu em 30 segundos. Tente de novo.",
    )
}

pub fn erro_chave_invalida() -> ErroApp {
    ErroApp::novo(
        "chave_invalida",
        "O Groq recusou a chave. Confira em Configurações › Provedores e chaves.",
    )
}

pub fn erro_do_provedor(status: u16) -> ErroApp {
    ErroApp::novo(
        "erro_provedor",
        &format!("O Groq respondeu com o erro {status}. Tente de novo mais tarde."),
    )
}

/// Sem chave cadastrada o Groq não funciona: avisa antes de gravar (RF-04.1, F-02).
pub fn exigir_chave(cofre: &dyn Cofre, config: &ConfigTranscricao) -> Result<(), ErroApp> {
    if config.provedor == ProvedorTranscricao::Groq {
        chaves::servico::obter(cofre, Provedor::Groq)?;
    }
    Ok(())
}

/// Fábrica real: Groq com a chave do cofre; o Whisper local entra com o F-22.
pub fn fabrica_com_cofre(cofre: Arc<dyn Cofre>) -> FabricaCompartilhada {
    FabricaCompartilhada(Arc::new(move |config| match config.provedor {
        ProvedorTranscricao::Groq => {
            let chave = chaves::servico::obter(cofre.as_ref(), Provedor::Groq)?;
            Ok(Box::new(groq::Groq::novo(chaves::groq::URL_BASE, &chave)) as Box<dyn Transcritor>)
        }
        ProvedorTranscricao::WhisperLocal => Err(ErroApp::novo(
            "provedor_indisponivel",
            "A transcrição local ainda não está disponível. Escolha o Groq neste modo.",
        )),
    }))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::chaves::cofre::CofreMemoria;

    #[test]
    fn groq_sem_chave_pede_para_cadastrar() {
        let cofre = CofreMemoria::default();
        let e = exigir_chave(&cofre, &ConfigTranscricao::default()).unwrap_err();
        assert_eq!(e.codigo, "chave_ausente");
        assert!(e.mensagem.starts_with("Cadastre a chave do Groq"));
        cofre.gravar(Provedor::Groq, "gsk_x").unwrap();
        assert!(exigir_chave(&cofre, &ConfigTranscricao::default()).is_ok());
    }

    #[test]
    fn fabrica_sem_chave_falha_e_local_ainda_indisponivel() {
        let fabrica = fabrica_com_cofre(Arc::new(CofreMemoria::default()));
        let groq = ConfigTranscricao::default();
        assert_eq!((fabrica.0)(&groq).err().unwrap().codigo, "chave_ausente");
        let local = ConfigTranscricao {
            provedor: ProvedorTranscricao::WhisperLocal,
            ..groq
        };
        assert_eq!(
            (fabrica.0)(&local).err().unwrap().codigo,
            "provedor_indisponivel"
        );
    }

    #[test]
    fn transcrever_devolve_texto_aparado() {
        struct Eco;
        impl Transcritor for Eco {
            fn transcrever(&self, _: &Pedido) -> Result<String, ErroApp> {
                Ok("  oi  ".into())
            }
        }
        let g = Gravacao {
            taxa_hz: 16_000,
            amostras: vec![9; 800],
        };
        assert_eq!(
            transcrever(&Eco, &ConfigTranscricao::default(), &g, None).unwrap(),
            "oi"
        );
    }
}

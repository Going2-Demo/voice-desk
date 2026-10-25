//! Pastas do app. Em desenvolvimento, `VOICE_DESK_PASTA_DADOS` separa os dados.

use std::path::{Path, PathBuf};

pub struct Caminhos {
    raiz: PathBuf,
}

impl Caminhos {
    pub fn em(raiz: impl Into<PathBuf>) -> Self {
        Self { raiz: raiz.into() }
    }

    /// Usa `VOICE_DESK_PASTA_DADOS` se definida; senão a pasta de dados do app no sistema.
    pub fn resolver(pasta_do_sistema: &Path) -> Self {
        match std::env::var_os("VOICE_DESK_PASTA_DADOS") {
            Some(p) if !p.is_empty() => Self::em(p),
            _ => Self::em(pasta_do_sistema),
        }
    }

    pub fn raiz(&self) -> &Path {
        &self.raiz
    }

    pub fn banco(&self) -> PathBuf {
        self.raiz.join("voice-desk.db")
    }
    pub fn logs(&self) -> PathBuf {
        self.raiz.join("logs")
    }
    pub fn audios(&self) -> PathBuf {
        self.raiz.join("audios")
    }
    pub fn modelos(&self) -> PathBuf {
        self.raiz.join("modelos")
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn nomes_seguem_a_arquitetura() {
        let c = Caminhos::em("/dados");
        assert_eq!(c.banco(), PathBuf::from("/dados/voice-desk.db"));
        assert_eq!(c.logs(), PathBuf::from("/dados/logs"));
        assert_eq!(c.audios(), PathBuf::from("/dados/audios"));
        assert_eq!(c.modelos(), PathBuf::from("/dados/modelos"));
    }
}

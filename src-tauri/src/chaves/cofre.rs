//! Cofre de chaves em arquivo cifrado no app (ENG-102, ADR-10): o mesmo modelo no Mac e no
//! Windows, sem Keychain e sem pedido de senha do sistema. O trait existe para os testes
//! usarem um cofre em memória.

use super::Provedor;
use crate::nucleo::ErroApp;
use chacha20poly1305::aead::{Aead, KeyInit};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use hkdf::Hkdf;
use sha2::Sha256;
use std::collections::{BTreeMap, HashMap};
use std::path::PathBuf;
use std::sync::Mutex;

pub const ARQUIVO: &str = "chaves.vdk";
const VERSAO: u8 = 1;
const TAM_NONCE: usize = 24;
const CONTEXTO: &[u8] = b"voice-desk/chaves/v1";

pub trait Cofre: Send + Sync {
    fn ler(&self, provedor: Provedor) -> Result<Option<String>, ErroApp>;
    fn gravar(&self, provedor: Provedor, chave: &str) -> Result<(), ErroApp>;
    /// Apagar uma chave que não existe não é erro.
    fn apagar(&self, provedor: Provedor) -> Result<(), ErroApp>;
}

fn erro_cofre(detalhe: &str) -> ErroApp {
    // O detalhe nunca carrega a chave; a interface só recebe a mensagem fixa.
    tracing::error!(evento = "erro_interno", origem = "cofre", detalhe = %detalhe);
    ErroApp::novo(
        "cofre_indisponivel",
        "Não consegui ler ou gravar suas chaves neste computador. Cole a chave de novo e tente outra vez.",
    )
}

/// Arquivo `chaves.vdk` na pasta de dados: versão (1 byte) + nonce (24) + texto cifrado com
/// XChaCha20-Poly1305 de um JSON `{conta: chave}`. A chave de cifragem vem do identificador
/// da máquina (HKDF-SHA256), então o arquivo copiado para outro computador não abre. Isso
/// protege contra leitura casual, backup e sincronização do arquivo; não contra um programa
/// rodando como o mesmo usuário na mesma máquina.
pub struct CofreArquivo {
    arquivo: PathBuf,
    segredo: Vec<u8>,
    trava: Mutex<()>,
}

impl CofreArquivo {
    pub fn novo(pasta: &std::path::Path) -> Result<Self, ErroApp> {
        let id = machine_uid::get().map_err(|e| erro_cofre(&e.to_string()))?;
        Ok(Self::com_segredo(pasta.join(ARQUIVO), id.into_bytes()))
    }

    pub fn com_segredo(arquivo: PathBuf, segredo: Vec<u8>) -> Self {
        Self {
            arquivo,
            segredo,
            trava: Mutex::new(()),
        }
    }

    fn cifra(&self) -> Result<XChaCha20Poly1305, ErroApp> {
        let mut chave = [0u8; 32];
        Hkdf::<Sha256>::new(Some(b"voice-desk"), &self.segredo)
            .expand(CONTEXTO, &mut chave)
            .map_err(|e| erro_cofre(&e.to_string()))?;
        XChaCha20Poly1305::new_from_slice(&chave).map_err(|e| erro_cofre(&e.to_string()))
    }

    fn carregar(&self) -> Result<BTreeMap<String, String>, ErroApp> {
        let bytes = match std::fs::read(&self.arquivo) {
            Ok(b) => b,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(e) => return Err(erro_cofre(&e.to_string())),
        };
        if bytes.len() <= 1 + TAM_NONCE || bytes[0] != VERSAO {
            return Err(erro_cofre("arquivo de chaves inválido"));
        }
        let (nonce, cifrado) = bytes[1..].split_at(TAM_NONCE);
        let nonce = XNonce::try_from(nonce).map_err(|_| erro_cofre("nonce inválido"))?;
        let claro = self
            .cifra()?
            .decrypt(&nonce, cifrado)
            .map_err(|_| erro_cofre("não foi possível decifrar o arquivo de chaves"))?;
        serde_json::from_slice(&claro).map_err(|e| erro_cofre(&e.to_string()))
    }

    fn salvar(&self, mapa: &BTreeMap<String, String>) -> Result<(), ErroApp> {
        let claro = serde_json::to_vec(mapa).map_err(|e| erro_cofre(&e.to_string()))?;
        let mut nonce = [0u8; TAM_NONCE];
        getrandom::fill(&mut nonce).map_err(|e| erro_cofre(&e.to_string()))?;
        let cifrado = self
            .cifra()?
            .encrypt(&XNonce::from(nonce), claro.as_slice())
            .map_err(|_| erro_cofre("falha ao cifrar"))?;
        let mut saida = Vec::with_capacity(1 + TAM_NONCE + cifrado.len());
        saida.push(VERSAO);
        saida.extend_from_slice(&nonce);
        saida.extend_from_slice(&cifrado);
        if let Some(pasta) = self.arquivo.parent() {
            std::fs::create_dir_all(pasta).map_err(|e| erro_cofre(&e.to_string()))?;
        }
        // Grava num temporário e renomeia: uma queda no meio não corrompe as chaves.
        let tmp = self.arquivo.with_extension("tmp");
        std::fs::write(&tmp, saida).map_err(|e| erro_cofre(&e.to_string()))?;
        restringir_permissao(&tmp);
        std::fs::rename(&tmp, &self.arquivo).map_err(|e| erro_cofre(&e.to_string()))
    }
}

#[cfg(unix)]
fn restringir_permissao(caminho: &std::path::Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(caminho, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restringir_permissao(_: &std::path::Path) {}

impl Cofre for CofreArquivo {
    fn ler(&self, provedor: Provedor) -> Result<Option<String>, ErroApp> {
        let _g = self.trava.lock().unwrap();
        Ok(self.carregar()?.get(provedor.conta()).cloned())
    }

    fn gravar(&self, provedor: Provedor, chave: &str) -> Result<(), ErroApp> {
        let _g = self.trava.lock().unwrap();
        let mut mapa = self.carregar()?;
        mapa.insert(provedor.conta().to_string(), chave.to_string());
        self.salvar(&mapa)
    }

    fn apagar(&self, provedor: Provedor) -> Result<(), ErroApp> {
        let _g = self.trava.lock().unwrap();
        let mut mapa = self.carregar()?;
        if mapa.remove(provedor.conta()).is_some() {
            self.salvar(&mapa)?;
        }
        Ok(())
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

    fn cofre(pasta: &std::path::Path, segredo: &str) -> CofreArquivo {
        CofreArquivo::com_segredo(pasta.join(ARQUIVO), segredo.as_bytes().to_vec())
    }

    #[test]
    fn arquivo_grava_le_e_apaga() {
        let d = tempfile::tempdir().unwrap();
        let c = cofre(d.path(), "maquina-a");
        assert_eq!(c.ler(Provedor::Groq).unwrap(), None);
        c.apagar(Provedor::Groq).unwrap();
        c.gravar(Provedor::Groq, "gsk_abc").unwrap();
        assert_eq!(c.ler(Provedor::Groq).unwrap().as_deref(), Some("gsk_abc"));
        c.gravar(Provedor::Groq, "gsk_nova").unwrap();
        assert_eq!(c.ler(Provedor::Groq).unwrap().as_deref(), Some("gsk_nova"));
        // Persiste: outra instância lê o mesmo arquivo.
        let outra = cofre(d.path(), "maquina-a");
        assert_eq!(
            outra.ler(Provedor::Groq).unwrap().as_deref(),
            Some("gsk_nova")
        );
        c.apagar(Provedor::Groq).unwrap();
        assert_eq!(c.ler(Provedor::Groq).unwrap(), None);
    }

    #[test]
    fn arquivo_nao_guarda_a_chave_em_texto_puro() {
        let d = tempfile::tempdir().unwrap();
        cofre(d.path(), "m")
            .gravar(Provedor::Groq, "gsk_segredo_1234")
            .unwrap();
        let bytes = std::fs::read(d.path().join(ARQUIVO)).unwrap();
        let texto = String::from_utf8_lossy(&bytes);
        assert!(!texto.contains("gsk_segredo_1234"));
        assert!(!texto.contains("groq"));
        assert_eq!(bytes[0], VERSAO);
    }

    #[test]
    fn arquivo_de_outra_maquina_nao_abre() {
        let d = tempfile::tempdir().unwrap();
        cofre(d.path(), "maquina-a")
            .gravar(Provedor::Groq, "gsk_abc")
            .unwrap();
        let e = cofre(d.path(), "maquina-b")
            .ler(Provedor::Groq)
            .unwrap_err();
        assert_eq!(e.codigo, "cofre_indisponivel");
        assert!(!e.mensagem.contains("gsk_abc"));
    }

    #[test]
    fn arquivo_adulterado_ou_truncado_da_erro() {
        let d = tempfile::tempdir().unwrap();
        let c = cofre(d.path(), "m");
        c.gravar(Provedor::Groq, "gsk_abc").unwrap();
        let caminho = d.path().join(ARQUIVO);
        let mut bytes = std::fs::read(&caminho).unwrap();
        let ultimo = bytes.len() - 1;
        bytes[ultimo] ^= 0xff;
        std::fs::write(&caminho, &bytes).unwrap();
        assert!(c.ler(Provedor::Groq).is_err());
        std::fs::write(&caminho, [1u8, 2, 3]).unwrap();
        assert!(c.ler(Provedor::Groq).is_err());
    }

    #[test]
    fn cada_gravacao_usa_nonce_novo() {
        let d = tempfile::tempdir().unwrap();
        let c = cofre(d.path(), "m");
        c.gravar(Provedor::Groq, "gsk_abc").unwrap();
        let a = std::fs::read(d.path().join(ARQUIVO)).unwrap();
        c.gravar(Provedor::Groq, "gsk_abc").unwrap();
        let b = std::fs::read(d.path().join(ARQUIVO)).unwrap();
        assert_ne!(a[1..25], b[1..25]);
    }

    #[cfg(unix)]
    #[test]
    fn arquivo_so_o_dono_le() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        cofre(d.path(), "m")
            .gravar(Provedor::Groq, "gsk_abc")
            .unwrap();
        let modo = std::fs::metadata(d.path().join(ARQUIVO))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(modo & 0o777, 0o600);
    }
}

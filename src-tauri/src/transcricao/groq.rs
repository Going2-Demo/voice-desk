//! Transcrição no Groq: `POST /openai/v1/audio/transcriptions` (multipart), limite de 30 s.

use super::{
    LIMITE_RESPOSTA_S, Pedido, Transcritor, erro_chave_invalida, erro_do_provedor,
    erro_sem_conexao, erro_tempo_esgotado,
};
use crate::nucleo::ErroApp;
use reqwest::blocking::{Client, multipart};
use std::time::Duration;

pub struct Groq {
    url_base: String,
    chave: String,
    limite: Duration,
}

impl Groq {
    pub fn novo(url_base: &str, chave: &str) -> Self {
        Self {
            url_base: url_base.trim_end_matches('/').into(),
            chave: chave.into(),
            limite: Duration::from_secs(LIMITE_RESPOSTA_S),
        }
    }

    #[cfg(test)]
    fn com_limite(mut self, limite: Duration) -> Self {
        self.limite = limite;
        self
    }
}

impl Transcritor for Groq {
    fn transcrever(&self, pedido: &Pedido) -> Result<String, ErroApp> {
        let cliente = Client::builder()
            .timeout(self.limite)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(ErroApp::interno)?;
        let arquivo = multipart::Part::bytes(pedido.audio_flac.clone())
            .file_name("audio.flac")
            .mime_str("audio/flac")
            .map_err(ErroApp::interno)?;
        let mut form = multipart::Form::new()
            .part("file", arquivo)
            .text("model", pedido.modelo.clone())
            .text("response_format", "json");
        if let Some(iso) = pedido.idioma.codigo_iso() {
            form = form.text("language", iso);
        }
        if let Some(dica) = pedido.dica.as_deref().filter(|d| !d.is_empty()) {
            form = form.text("prompt", dica.to_string());
        }
        let resposta = cliente
            .post(format!("{}/openai/v1/audio/transcriptions", self.url_base))
            .bearer_auth(&self.chave)
            .multipart(form)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    erro_tempo_esgotado()
                } else {
                    erro_sem_conexao()
                }
            })?;
        let status = resposta.status().as_u16();
        match status {
            200..=299 => {
                let json: serde_json::Value = resposta.json().map_err(|e| {
                    if e.is_timeout() {
                        erro_tempo_esgotado()
                    } else {
                        erro_do_provedor(status)
                    }
                })?;
                json.get("text")
                    .and_then(|t| t.as_str())
                    .map(str::to_string)
                    .ok_or_else(|| erro_do_provedor(status))
            }
            401 | 403 => Err(erro_chave_invalida()),
            _ => Err(erro_do_provedor(status)),
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::transcricao::Idioma;
    use std::io::{Read, Write};
    use std::net::TcpListener;

    /// Servidor mínimo: lê a requisição inteira, responde `resposta` e devolve o que leu.
    fn servidor(resposta: &'static str) -> (String, std::thread::JoinHandle<String>) {
        let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", ouvinte.local_addr().unwrap());
        let fio = std::thread::spawn(move || {
            let (mut s, _) = ouvinte.accept().unwrap();
            let mut lido = Vec::new();
            let mut buf = [0u8; 4096];
            loop {
                let n = s.read(&mut buf).unwrap_or(0);
                lido.extend_from_slice(&buf[..n]);
                let t = String::from_utf8_lossy(&lido);
                if n == 0 || terminou(&t) {
                    break;
                }
            }
            s.write_all(resposta.as_bytes()).unwrap();
            String::from_utf8_lossy(&lido).into_owned()
        });
        (url, fio)
    }

    fn terminou(t: &str) -> bool {
        t.trim_end().ends_with("--")
    }

    fn pedido(idioma: Idioma) -> Pedido {
        Pedido {
            audio_flac: b"fLaCxxxx".to_vec(),
            modelo: "whisper-large-v3-turbo".into(),
            idioma,
            dica: None,
        }
    }

    fn http(status: &str, corpo: &str) -> &'static str {
        Box::leak(
            format!(
                "HTTP/1.1 {status}\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{corpo}",
                corpo.len()
            )
            .into_boxed_str(),
        )
    }

    #[test]
    fn devolve_o_texto_e_envia_chave_modelo_e_idioma() {
        let (url, fio) = servidor(http("200 OK", r#"{"text":" olá mundo "}"#));
        let t = Groq::novo(&url, "gsk_segredo")
            .transcrever(&pedido(Idioma::En))
            .unwrap();
        assert_eq!(t, " olá mundo ");
        let req = fio.join().unwrap();
        assert!(req.starts_with("POST /openai/v1/audio/transcriptions"));
        assert!(
            req.to_lowercase()
                .contains("authorization: bearer gsk_segredo")
        );
        assert!(req.contains("whisper-large-v3-turbo"));
        assert!(req.contains("name=\"language\"\r\n\r\nen"));
    }

    #[test]
    fn idioma_automatico_omite_language() {
        let (url, fio) = servidor(http("200 OK", r#"{"text":"x"}"#));
        Groq::novo(&url, "k")
            .transcrever(&pedido(Idioma::Auto))
            .unwrap();
        assert!(!fio.join().unwrap().contains("name=\"language\""));
    }

    #[test]
    fn chave_recusada() {
        let (url, _f) = servidor(http("401 Unauthorized", r#"{"error":{"message":"x"}}"#));
        let e = Groq::novo(&url, "k")
            .transcrever(&pedido(Idioma::PtBr))
            .unwrap_err();
        assert_eq!(e.codigo, "chave_invalida");
    }

    #[test]
    fn erro_do_servidor() {
        let (url, _f) = servidor(http("500 Internal Server Error", "{}"));
        let e = Groq::novo(&url, "k")
            .transcrever(&pedido(Idioma::PtBr))
            .unwrap_err();
        assert_eq!(e.codigo, "erro_provedor");
        assert!(e.mensagem.contains("500"));
    }

    #[test]
    fn sem_resposta_no_limite_avisa_tempo_esgotado() {
        let ouvinte = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", ouvinte.local_addr().unwrap());
        let _mudo = std::thread::spawn(move || {
            let _conexao = ouvinte.accept();
            std::thread::sleep(Duration::from_secs(3));
        });
        let e = Groq::novo(&url, "k")
            .com_limite(Duration::from_millis(300))
            .transcrever(&pedido(Idioma::PtBr))
            .unwrap_err();
        assert_eq!(e.codigo, "tempo_esgotado");
    }

    #[test]
    fn sem_rede_avisa_sem_conexao() {
        // Porta fechada: ninguém escuta.
        let porta = TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let e = Groq::novo(&format!("http://127.0.0.1:{porta}"), "k")
            .transcrever(&pedido(Idioma::PtBr))
            .unwrap_err();
        assert_eq!(e.codigo, "sem_conexao");
        assert!(e.mensagem.starts_with("Sem conexão com o provedor"));
    }
}

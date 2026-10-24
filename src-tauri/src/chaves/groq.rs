//! Teste da chave no Groq: `GET /openai/v1/models`, limite de 10 s (RF-04.2).

use super::{ResultadoTeste, Validade};
use std::time::Duration;

pub const URL_BASE: &str = "https://api.groq.com";
const LIMITE: Duration = Duration::from_secs(10);
const MAX_MOTIVO: usize = 200;

pub async fn testar(url_base: &str, chave: &str) -> ResultadoTeste {
    let cliente = match reqwest::Client::builder()
        .timeout(LIMITE)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(c) => c,
        Err(_) => return nao_testada("Não consegui preparar a conexão com o Groq."),
    };
    let resposta = cliente
        .get(format!("{url_base}/openai/v1/models"))
        .bearer_auth(chave)
        .send()
        .await;
    match resposta {
        Err(e) if e.is_timeout() => {
            nao_testada("O Groq não respondeu em 10 segundos. Tente de novo.")
        }
        Err(_) => nao_testada("Não consegui falar com o Groq. Confira a conexão com a internet."),
        Ok(r) if r.status().is_success() => ResultadoTeste {
            validade: Validade::Valida,
            motivo: None,
        },
        Ok(r) if matches!(r.status().as_u16(), 401 | 403) => {
            let status = r.status().as_u16();
            let corpo = r.text().await.unwrap_or_default();
            ResultadoTeste {
                validade: Validade::Invalida,
                motivo: Some(
                    motivo_do_provedor(&corpo, chave)
                        .unwrap_or_else(|| format!("O Groq recusou a chave (erro {status}).")),
                ),
            }
        }
        Ok(r) => nao_testada(&format!(
            "O Groq respondeu com o erro {}. Tente de novo mais tarde.",
            r.status().as_u16()
        )),
    }
}

fn nao_testada(motivo: &str) -> ResultadoTeste {
    ResultadoTeste {
        validade: Validade::NaoTestada,
        motivo: Some(motivo.into()),
    }
}

/// Extrai `error.message` da resposta do Groq, tirando a chave se o texto a repetir.
fn motivo_do_provedor(corpo: &str, chave: &str) -> Option<String> {
    let json: serde_json::Value = serde_json::from_str(corpo).ok()?;
    let texto = json.get("error")?.get("message")?.as_str()?.trim();
    if texto.is_empty() {
        return None;
    }
    let limpo: String = texto
        .replace(chave, "••••")
        .chars()
        .take(MAX_MOTIVO)
        .collect();
    Some(limpo)
}

#[cfg(test)]
mod testes {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    /// Servidor HTTP mínimo: responde `resposta` a uma conexão e devolve a requisição lida.
    async fn servidor(resposta: &'static str) -> (String, tokio::task::JoinHandle<String>) {
        let ouvinte = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let url = format!("http://{}", ouvinte.local_addr().unwrap());
        let tarefa = tokio::spawn(async move {
            let (mut s, _) = ouvinte.accept().await.unwrap();
            let mut buf = vec![0u8; 4096];
            let n = s.read(&mut buf).await.unwrap();
            s.write_all(resposta.as_bytes()).await.unwrap();
            s.shutdown().await.ok();
            String::from_utf8_lossy(&buf[..n]).into_owned()
        });
        (url, tarefa)
    }

    #[tokio::test]
    async fn chave_aceita_e_valida_e_vai_no_cabecalho() {
        let (url, req) = servidor(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: 11\r\nconnection: close\r\n\r\n{\"data\":[]}",
        )
        .await;
        let r = testar(&url, "gsk_valida").await;
        assert_eq!(r.validade, Validade::Valida);
        assert_eq!(r.motivo, None);
        let req = req.await.unwrap().to_lowercase();
        assert!(req.starts_with("get /openai/v1/models"));
        assert!(req.contains("authorization: bearer gsk_valida"));
    }

    #[tokio::test]
    async fn chave_inventada_e_invalida_com_o_motivo_do_provedor() {
        let corpo = r#"{"error":{"message":"Invalid API Key","type":"invalid_request_error"}}"#;
        let resposta: &'static str = Box::leak(
            format!(
                "HTTP/1.1 401 Unauthorized\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{corpo}",
                corpo.len()
            )
            .into_boxed_str(),
        );
        let (url, _) = servidor(resposta).await;
        let r = testar(&url, "gsk_inventada").await;
        assert_eq!(r.validade, Validade::Invalida);
        assert_eq!(r.motivo.as_deref(), Some("Invalid API Key"));
    }

    #[tokio::test]
    async fn motivo_nao_repete_a_chave() {
        let m = motivo_do_provedor(
            r#"{"error":{"message":"bad key gsk_segredo123 rejected"}}"#,
            "gsk_segredo123",
        )
        .unwrap();
        assert_eq!(m, "bad key •••• rejected");
    }

    #[tokio::test]
    async fn erro_do_provedor_nao_marca_invalida() {
        let (url, _) = servidor(
            "HTTP/1.1 503 Service Unavailable\r\ncontent-length: 0\r\nconnection: close\r\n\r\n",
        )
        .await;
        let r = testar(&url, "gsk_x").await;
        assert_eq!(r.validade, Validade::NaoTestada);
        assert!(r.motivo.unwrap().contains("503"));
    }

    #[tokio::test]
    async fn sem_conexao_fica_nao_testada() {
        // Porta fechada: abre e solta o ouvinte para pegar uma porta livre.
        let porta = TcpListener::bind("127.0.0.1:0")
            .await
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let r = testar(&format!("http://127.0.0.1:{porta}"), "gsk_x").await;
        assert_eq!(r.validade, Validade::NaoTestada);
    }
}

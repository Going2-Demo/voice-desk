//! Etapa `transcrever` (F-02): envia o áudio ao provedor do modo ativo e guarda o texto. Sem
//! áudio, passa adiante. Em falha devolve o erro com a mensagem pronta (RF-02.3); quem chamou o
//! pipeline guarda o áudio no histórico como "falhou" e o gravador mostra o motivo.

use super::{Contexto, Fluxo};
use crate::nucleo::ErroApp;
use crate::transcricao;

pub fn executar(contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    let Some(gravacao) = contexto.gravacao.as_ref() else {
        return Ok(Fluxo::Seguir);
    };
    let fabrica = contexto
        .fabrica_transcritor
        .as_ref()
        .ok_or_else(|| ErroApp::interno("transcrição sem fábrica de provedor"))?;
    let transcritor = (fabrica.0)(&contexto.config_transcricao)?;
    let texto = transcricao::transcrever(
        transcritor.as_ref(),
        &contexto.config_transcricao,
        gravacao,
        None,
    )?;
    tracing::info!(
        evento = "transcricao_concluida",
        vazio = texto.is_empty(),
        amostras = gravacao.amostras.len(),
        taxa_hz = gravacao.taxa_hz
    );
    // As etapas seguintes (substituir, limpar, refinar) transformam `texto_final`; sem elas, é o transcrito.
    contexto.texto_final = texto.clone();
    contexto.texto_transcrito = texto;
    contexto.transcricao_provedor = Some(contexto.config_transcricao.provedor);
    contexto.transcricao_modelo = Some(contexto.config_transcricao.modelo.clone());
    Ok(Fluxo::Seguir)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::audio::captura::Gravacao;
    use crate::transcricao::{
        ConfigTranscricao, FabricaCompartilhada, Idioma, Pedido, ProvedorTranscricao, Transcritor,
        erro_sem_conexao,
    };
    use std::sync::{Arc, Mutex};

    type Pedidos = Arc<Mutex<Vec<(String, Idioma)>>>;

    struct Falso {
        resposta: Result<String, ErroApp>,
        pedidos: Pedidos,
    }

    impl Transcritor for Falso {
        fn transcrever(&self, p: &Pedido) -> Result<String, ErroApp> {
            self.pedidos
                .lock()
                .unwrap()
                .push((p.modelo.clone(), p.idioma));
            self.resposta.clone()
        }
    }

    fn contexto(
        resposta: Result<String, ErroApp>,
        config: ConfigTranscricao,
    ) -> (Contexto, Pedidos) {
        let pedidos = Arc::new(Mutex::new(Vec::new()));
        let p = Arc::clone(&pedidos);
        let fabrica = FabricaCompartilhada(Arc::new(move |_| {
            Ok(Box::new(Falso {
                resposta: resposta.clone(),
                pedidos: Arc::clone(&p),
            }) as Box<dyn Transcritor>)
        }));
        let c = Contexto {
            gravacao: Some(Gravacao {
                taxa_hz: 16_000,
                amostras: vec![500; 1600],
            }),
            config_transcricao: config,
            fabrica_transcritor: Some(fabrica),
            ..Contexto::default()
        };
        (c, pedidos)
    }

    #[test]
    fn guarda_texto_provedor_e_modelo_do_modo() {
        let cfg = ConfigTranscricao {
            provedor: ProvedorTranscricao::Groq,
            modelo: "whisper-large-v3".into(),
            idioma: Idioma::En,
        };
        let (mut c, pedidos) = contexto(Ok("  hello  ".into()), cfg);
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
        assert_eq!(c.texto_transcrito, "hello");
        assert_eq!(c.texto_final, "hello");
        assert_eq!(c.transcricao_provedor, Some(ProvedorTranscricao::Groq));
        assert_eq!(c.transcricao_modelo.as_deref(), Some("whisper-large-v3"));
        assert_eq!(
            *pedidos.lock().unwrap(),
            [("whisper-large-v3".to_string(), Idioma::En)]
        );
    }

    #[test]
    fn falha_do_provedor_sobe_com_o_motivo_e_nao_deixa_texto() {
        let (mut c, _) = contexto(Err(erro_sem_conexao()), ConfigTranscricao::default());
        let e = executar(&mut c).unwrap_err();
        assert_eq!(e.codigo, "sem_conexao");
        assert!(c.texto_transcrito.is_empty());
    }

    #[test]
    fn chave_ausente_na_fabrica_sobe_antes_de_enviar() {
        let mut c = Contexto {
            gravacao: Some(Gravacao {
                taxa_hz: 16_000,
                amostras: vec![1; 100],
            }),
            fabrica_transcritor: Some(FabricaCompartilhada(Arc::new(|_| {
                Err(ErroApp::novo("chave_ausente", "Cadastre a chave do Groq"))
            }))),
            ..Contexto::default()
        };
        assert_eq!(executar(&mut c).unwrap_err().codigo, "chave_ausente");
    }

    #[test]
    fn sem_audio_passa_adiante() {
        let mut c = Contexto::default();
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
    }
}

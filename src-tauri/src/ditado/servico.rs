//! Orquestrador do ditado (F-01): liga a máquina de estados aos efeitos reais (gravador, áudio,
//! foco, teclado). Os efeitos entram por `Efeitos`, então tudo aqui é testável sem sistema.

use super::colagem::ResultadoColagem;
use super::etapas::Contexto;
use super::maquina::{Acao, Fase, Maquina};
use crate::audio::captura::Gravacao;
use crate::nucleo::ErroApp;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::{Arc, Mutex};

/// O que o gravador mostra. `fase`: `gravando`, `processando`, `ok`, `aviso` ou `erro`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct EstadoGravador {
    pub fase: String,
    pub mensagem: String,
    pub segundos: u32,
    /// Programa que estava em foco quando o ditado começou.
    pub programa: Option<String>,
    /// Atalho oferecido no gravador: `abrir_chaves` (sem chave do provedor, RF-04.1).
    pub acao: Option<String>,
}

pub const MSG_GRAVANDO: &str = "Gravando… acione o atalho de novo para parar";
pub const MSG_PROCESSANDO: &str = "Processando…";
pub const MSG_AGUARDE: &str = "Aguarde: ainda processando";
pub const MSG_LIMITE: &str = "Parando automaticamente em 30 s";
pub const MSG_PRONTO: &str = "Pronto";
pub const MSG_SEM_FALA: &str = "Não ouvi fala. Tente de novo, falando mais perto do microfone.";

/// Aviso de RF-03.4, com o atalho da plataforma.
pub fn mensagem_nao_colado() -> String {
    format!(
        "Texto copiado; cole com {}",
        crate::plataforma::colagem::atalho_manual()
    )
}

pub trait Efeitos: Send + Sync + 'static {
    fn agora_ms(&self) -> u64;
    /// Mostra o gravador sem tirar o foco do programa atual.
    fn mostrar_gravador(&self);
    fn ocultar_gravador(&self, depois_ms: u64);
    fn emitir_estado(&self, estado: &EstadoGravador);
    fn programa_em_foco(&self) -> Option<String>;
    fn iniciar_captura(&self) -> Result<(), ErroApp>;
    /// Para a captura e devolve o áudio.
    fn parar_captura(&self) -> Option<Gravacao>;
    /// Para a captura e joga o áudio fora.
    fn descartar_captura(&self);
    /// Maior amplitude capturada até agora, se a captura está aberta.
    fn pico_da_captura(&self) -> Option<f32>;
    fn escutar_esc(&self, ligar: bool);
    /// Confere, antes de gravar, o que a transcrição do modo ativo exige (ex.: chave cadastrada).
    fn conferir_transcricao(&self) -> Result<(), ErroApp>;
    /// Abre a tela de chaves (atalho do aviso "cadastre a chave").
    fn abrir_chaves(&self);
    /// Roda o processamento fora da thread atual e chama `ao_fim` quando termina.
    fn processar(
        &self,
        contexto: Contexto,
        gravacao: Option<Gravacao>,
        ao_fim: Box<dyn FnOnce(Result<Contexto, ErroApp>) + Send>,
    );
}

pub struct Ditado<E: Efeitos> {
    efeitos: E,
    maquina: Mutex<Maquina>,
    programa: Mutex<Option<String>>,
}

/// Tempo sem sinal algum no microfone que indica falta de permissão (macOS entrega silêncio).
const SILENCIO_DIGITAL_MS: u64 = 1500;
const OCULTAR_APOS_OK_MS: u64 = 900;
const OCULTAR_APOS_AVISO_MS: u64 = 4000;
const OCULTAR_APOS_AVISO_COM_ACAO_MS: u64 = 10_000;

impl<E: Efeitos> Ditado<E> {
    pub fn novo(efeitos: E) -> Arc<Self> {
        Arc::new(Self {
            efeitos,
            maquina: Mutex::new(Maquina::nova()),
            programa: Mutex::new(None),
        })
    }

    pub fn fase(&self) -> Fase {
        self.maquina
            .lock()
            .map(|m| m.fase())
            .unwrap_or(Fase::Ocioso)
    }

    fn estado(&self, fase: &str, mensagem: &str, segundos: u32) {
        let programa = self.programa.lock().ok().and_then(|p| p.clone());
        self.efeitos.emitir_estado(&EstadoGravador {
            fase: fase.into(),
            mensagem: mensagem.into(),
            segundos,
            programa,
            acao: None,
        });
    }

    /// O atalho do aviso no gravador foi clicado.
    pub fn abrir_chaves(&self) {
        self.efeitos.abrir_chaves();
    }

    /// O atalho global foi acionado.
    pub fn atalho(self: &Arc<Self>) {
        let agora = self.efeitos.agora_ms();
        let acao = self
            .maquina
            .lock()
            .map(|mut m| m.atalho(agora))
            .unwrap_or(Acao::Nenhuma);
        match acao {
            Acao::Iniciar => self.iniciar(),
            Acao::Parar | Acao::PararPorLimite => self.parar(),
            Acao::Aguarde => self.estado("processando", MSG_AGUARDE, 0),
            _ => {}
        }
    }

    pub fn esc(self: &Arc<Self>) {
        let agora = self.efeitos.agora_ms();
        let acao = self
            .maquina
            .lock()
            .map(|mut m| m.esc(agora))
            .unwrap_or(Acao::Nenhuma);
        if acao == Acao::Cancelar {
            self.efeitos.escutar_esc(false);
            self.efeitos.descartar_captura();
            self.efeitos.ocultar_gravador(0);
            *self.programa.lock().expect("programa") = None;
            tracing::info!(evento = "ditado_cancelado");
        }
    }

    /// Chamado a cada ~200 ms enquanto grava (relógio do gravador, limite de 10 min, aviso).
    pub fn tique(self: &Arc<Self>) {
        let agora = self.efeitos.agora_ms();
        let (acao, decorrido) = match self.maquina.lock() {
            Ok(mut m) => (m.tique(agora), m.decorrido_ms(agora)),
            Err(_) => return,
        };
        let segundos = (decorrido / 1000) as u32;
        match acao {
            Acao::PararPorLimite => self.parar(),
            Acao::AvisarLimite => self.estado("gravando", MSG_LIMITE, segundos),
            _ if self.fase() == Fase::Gravando => {
                if decorrido >= SILENCIO_DIGITAL_MS && self.efeitos.pico_da_captura() == Some(0.0) {
                    self.falhar_ao_iniciar(&crate::audio::captura::erro_sem_microfone());
                } else {
                    let aviso = decorrido >= super::maquina::AVISO_MS;
                    self.estado(
                        "gravando",
                        if aviso { MSG_LIMITE } else { MSG_GRAVANDO },
                        segundos,
                    );
                }
            }
            _ => {}
        }
    }

    fn iniciar(self: &Arc<Self>) {
        *self.programa.lock().expect("programa") = self.efeitos.programa_em_foco();
        if let Err(e) = self
            .efeitos
            .conferir_transcricao()
            .and_then(|()| self.efeitos.iniciar_captura())
        {
            self.falhar_ao_iniciar(&e);
            return;
        }
        self.efeitos.mostrar_gravador();
        self.efeitos.escutar_esc(true);
        self.estado("gravando", MSG_GRAVANDO, 0);
        tracing::info!(evento = "ditado_iniciado");
    }

    fn falhar_ao_iniciar(&self, erro: &ErroApp) {
        self.efeitos.escutar_esc(false);
        self.efeitos.descartar_captura();
        if let Ok(mut m) = self.maquina.lock() {
            m.falha_ao_iniciar();
        }
        self.efeitos.mostrar_gravador();
        let com_acao = erro.codigo == "chave_ausente";
        self.efeitos.emitir_estado(&EstadoGravador {
            fase: "erro".into(),
            mensagem: erro.mensagem.clone(),
            segundos: 0,
            programa: self.programa.lock().ok().and_then(|p| p.clone()),
            acao: com_acao.then(|| "abrir_chaves".to_string()),
        });
        self.efeitos.ocultar_gravador(if com_acao {
            OCULTAR_APOS_AVISO_COM_ACAO_MS
        } else {
            OCULTAR_APOS_AVISO_MS
        });
        *self.programa.lock().expect("programa") = None;
    }

    fn parar(self: &Arc<Self>) {
        self.efeitos.escutar_esc(false);
        let gravacao = self.efeitos.parar_captura();
        self.estado("processando", MSG_PROCESSANDO, 0);
        let contexto = Contexto {
            programa_em_foco: self.programa.lock().ok().and_then(|p| p.clone()),
            ..Contexto::default()
        };
        let este = Arc::clone(self);
        self.efeitos.processar(
            contexto,
            gravacao,
            Box::new(move |resultado| este.concluir(resultado)),
        );
    }

    fn concluir(&self, resultado: Result<Contexto, ErroApp>) {
        match resultado {
            Ok(contexto) if contexto.colagem == Some(ResultadoColagem::NaoColado) => {
                // RF-03.4: o texto ficou na área de transferência.
                self.estado("aviso", &mensagem_nao_colado(), 0);
                self.efeitos.ocultar_gravador(OCULTAR_APOS_AVISO_MS);
            }
            Ok(contexto)
                if contexto.transcricao_provedor.is_some() && contexto.texto_final.is_empty() =>
            {
                // Transcrição vazia: nada foi colado, então "Pronto" seria enganoso.
                self.estado("aviso", MSG_SEM_FALA, 0);
                self.efeitos.ocultar_gravador(OCULTAR_APOS_AVISO_MS);
            }
            Ok(_) => {
                self.estado("ok", MSG_PRONTO, 0);
                self.efeitos.ocultar_gravador(OCULTAR_APOS_OK_MS);
            }
            Err(e) => {
                self.estado("erro", &e.mensagem, 0);
                self.efeitos.ocultar_gravador(OCULTAR_APOS_AVISO_MS);
            }
        }
        if let Ok(mut m) = self.maquina.lock() {
            m.concluir();
        }
        *self.programa.lock().expect("programa") = None;
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    type Fim = Box<dyn FnOnce(Result<Contexto, ErroApp>) + Send>;

    #[derive(Default)]
    struct Falso {
        agora: AtomicU64,
        eventos: Mutex<Vec<String>>,
        estados: Mutex<Vec<EstadoGravador>>,
        sem_microfone: Mutex<bool>,
        sem_chave: Mutex<bool>,
        pico: Mutex<Option<f32>>,
        pendente: Mutex<Option<Fim>>,
        contexto: Mutex<Option<Contexto>>,
    }

    impl Falso {
        fn registra(&self, e: &str) {
            self.eventos.lock().unwrap().push(e.into());
        }
    }

    impl Efeitos for Arc<Falso> {
        fn agora_ms(&self) -> u64 {
            self.agora.load(Ordering::Relaxed)
        }
        fn mostrar_gravador(&self) {
            self.registra("mostrar");
        }
        fn ocultar_gravador(&self, depois: u64) {
            self.registra(&format!("ocultar:{depois}"));
        }
        fn emitir_estado(&self, e: &EstadoGravador) {
            self.estados.lock().unwrap().push(e.clone());
        }
        fn programa_em_foco(&self) -> Option<String> {
            Some("Bloco de Notas".into())
        }
        fn iniciar_captura(&self) -> Result<(), ErroApp> {
            self.registra("captura:iniciar");
            if *self.sem_microfone.lock().unwrap() {
                return Err(crate::audio::captura::erro_sem_microfone());
            }
            Ok(())
        }
        fn parar_captura(&self) -> Option<Gravacao> {
            self.registra("captura:parar");
            Some(Gravacao::default())
        }
        fn descartar_captura(&self) {
            self.registra("captura:descartar");
        }
        fn pico_da_captura(&self) -> Option<f32> {
            *self.pico.lock().unwrap()
        }
        fn escutar_esc(&self, ligar: bool) {
            self.registra(&format!("esc:{ligar}"));
        }
        fn conferir_transcricao(&self) -> Result<(), ErroApp> {
            if *self.sem_chave.lock().unwrap() {
                return Err(ErroApp::novo("chave_ausente", "Cadastre a chave do Groq"));
            }
            Ok(())
        }
        fn abrir_chaves(&self) {
            self.registra("abrir_chaves");
        }
        fn processar(&self, contexto: Contexto, _: Option<Gravacao>, ao_fim: Fim) {
            self.registra("processar");
            *self.contexto.lock().unwrap() = Some(contexto);
            *self.pendente.lock().unwrap() = Some(ao_fim);
        }
    }

    fn montar() -> (Arc<Falso>, Arc<Ditado<Arc<Falso>>>) {
        let f = Arc::new(Falso::default());
        (Arc::clone(&f), Ditado::novo(f))
    }

    fn terminar(f: &Falso) {
        let fim = f.pendente.lock().unwrap().take().unwrap();
        fim(Ok(Contexto::default()));
    }

    #[test]
    fn nao_colado_termina_com_aviso_e_o_atalho_manual() {
        let (f, d) = montar();
        d.atalho();
        d.atalho();
        let fim = f.pendente.lock().unwrap().take().unwrap();
        fim(Ok(Contexto {
            colagem: Some(ResultadoColagem::NaoColado),
            ..Contexto::default()
        }));
        let e = ultimo(&f);
        assert_eq!(e.fase, "aviso");
        assert_eq!(e.mensagem, mensagem_nao_colado());
        assert!(e.mensagem.starts_with("Texto copiado; cole com "));
        assert_eq!(eventos(&f).last().unwrap(), "ocultar:4000");
    }

    #[test]
    fn colado_termina_com_pronto() {
        let (f, d) = montar();
        d.atalho();
        d.atalho();
        let fim = f.pendente.lock().unwrap().take().unwrap();
        fim(Ok(Contexto {
            colagem: Some(ResultadoColagem::Colado),
            ..Contexto::default()
        }));
        assert_eq!(ultimo(&f).fase, "ok");
    }

    #[test]
    fn transcricao_vazia_avisa_em_vez_de_pronto() {
        let (f, d) = montar();
        d.atalho();
        d.atalho();
        let fim = f.pendente.lock().unwrap().take().unwrap();
        fim(Ok(Contexto {
            transcricao_provedor: Some(crate::transcricao::ProvedorTranscricao::Groq),
            ..Contexto::default()
        }));
        let e = ultimo(&f);
        assert_eq!(e.fase, "aviso");
        assert_eq!(e.mensagem, MSG_SEM_FALA);
    }

    fn eventos(f: &Falso) -> Vec<String> {
        f.eventos.lock().unwrap().clone()
    }

    fn ultimo(f: &Falso) -> EstadoGravador {
        f.estados.lock().unwrap().last().unwrap().clone()
    }

    #[test]
    fn sem_chave_avisa_antes_de_gravar_com_atalho_para_as_chaves() {
        let (f, d) = montar();
        *f.sem_chave.lock().unwrap() = true;
        d.atalho();
        assert!(!eventos(&f).contains(&"captura:iniciar".to_string()));
        let e = ultimo(&f);
        assert_eq!(e.fase, "erro");
        assert_eq!(e.mensagem, "Cadastre a chave do Groq");
        assert_eq!(e.acao.as_deref(), Some("abrir_chaves"));
        assert_eq!(d.fase(), Fase::Ocioso);
        d.abrir_chaves();
        assert_eq!(eventos(&f).last().unwrap(), "abrir_chaves");
    }

    #[test]
    fn fluxo_completo_mostra_grava_processa_e_some() {
        let (f, d) = montar();
        d.atalho();
        assert_eq!(eventos(&f), ["captura:iniciar", "mostrar", "esc:true"]);
        assert_eq!(ultimo(&f).fase, "gravando");
        assert_eq!(ultimo(&f).programa.as_deref(), Some("Bloco de Notas"));
        d.atalho();
        assert_eq!(ultimo(&f).fase, "processando");
        assert_eq!(d.fase(), Fase::Processando);
        terminar(&f);
        assert_eq!(ultimo(&f).fase, "ok");
        assert_eq!(eventos(&f).last().unwrap(), "ocultar:900");
        assert_eq!(d.fase(), Fase::Ocioso);
    }

    #[test]
    fn contexto_do_processamento_guarda_o_programa_em_foco() {
        let (f, d) = montar();
        d.atalho();
        d.atalho();
        let c = f.contexto.lock().unwrap().clone().unwrap();
        assert_eq!(c.programa_em_foco.as_deref(), Some("Bloco de Notas"));
    }

    #[test]
    fn atalho_durante_o_processamento_mostra_aguarde_e_nao_grava_de_novo() {
        let (f, d) = montar();
        d.atalho();
        d.atalho();
        d.atalho();
        assert_eq!(ultimo(&f).mensagem, MSG_AGUARDE);
        assert_eq!(
            eventos(&f)
                .iter()
                .filter(|e| *e == "captura:iniciar")
                .count(),
            1
        );
        assert_eq!(d.fase(), Fase::Processando);
    }

    #[test]
    fn esc_duas_vezes_cancela_sem_processar() {
        let (f, d) = montar();
        d.atalho();
        f.agora.store(1_000, Ordering::Relaxed);
        d.esc();
        assert_eq!(d.fase(), Fase::Gravando);
        f.agora.store(2_000, Ordering::Relaxed);
        d.esc();
        assert_eq!(d.fase(), Fase::Ocioso);
        let e = eventos(&f);
        assert!(e.contains(&"captura:descartar".to_string()));
        assert!(!e.contains(&"processar".to_string()));
        assert_eq!(e.last().unwrap(), "ocultar:0");
    }

    #[test]
    fn esc_uma_vez_nao_cancela() {
        let (f, d) = montar();
        d.atalho();
        d.esc();
        assert_eq!(d.fase(), Fase::Gravando);
        assert!(!eventos(&f).contains(&"captura:descartar".to_string()));
    }

    #[test]
    fn sem_permissao_do_microfone_mostra_o_erro_e_nada_e_gravado() {
        let (f, d) = montar();
        *f.sem_microfone.lock().unwrap() = true;
        d.atalho();
        let e = ultimo(&f);
        assert_eq!(e.fase, "erro");
        assert!(e.mensagem.contains("Sem acesso ao microfone"));
        assert!(e.mensagem.contains("Permissões"));
        assert_eq!(d.fase(), Fase::Ocioso);
        assert!(!eventos(&f).contains(&"processar".to_string()));
        d.atalho();
        assert_eq!(
            eventos(&f)
                .iter()
                .filter(|e| *e == "captura:iniciar")
                .count(),
            2
        );
    }

    #[test]
    fn silencio_digital_apos_1500ms_vira_erro_de_microfone() {
        let (f, d) = montar();
        d.atalho();
        *f.pico.lock().unwrap() = Some(0.0);
        f.agora.store(1_000, Ordering::Relaxed);
        d.tique();
        assert_eq!(d.fase(), Fase::Gravando);
        f.agora.store(1_600, Ordering::Relaxed);
        d.tique();
        assert_eq!(d.fase(), Fase::Ocioso);
        assert_eq!(ultimo(&f).fase, "erro");
    }

    #[test]
    fn tique_atualiza_o_tempo_avisa_aos_9min30_e_para_aos_10min() {
        let (f, d) = montar();
        d.atalho();
        *f.pico.lock().unwrap() = Some(0.2);
        f.agora.store(7_000, Ordering::Relaxed);
        d.tique();
        assert_eq!(ultimo(&f).segundos, 7);
        f.agora.store(570_000, Ordering::Relaxed);
        d.tique();
        assert_eq!(ultimo(&f).mensagem, MSG_LIMITE);
        f.agora.store(571_000, Ordering::Relaxed);
        d.tique();
        assert_eq!(ultimo(&f).mensagem, MSG_LIMITE);
        f.agora.store(600_000, Ordering::Relaxed);
        d.tique();
        assert_eq!(ultimo(&f).fase, "processando");
        assert_eq!(d.fase(), Fase::Processando);
        assert!(eventos(&f).contains(&"processar".to_string()));
    }

    #[test]
    fn falha_no_processamento_mostra_erro_e_libera_o_atalho() {
        let (f, d) = montar();
        d.atalho();
        d.atalho();
        let fim = f.pendente.lock().unwrap().take().unwrap();
        fim(Err(ErroApp::novo("falhou", "Falhou: sem conexão")));
        assert_eq!(ultimo(&f).fase, "erro");
        assert_eq!(d.fase(), Fase::Ocioso);
    }
}

//! Efeitos reais do ditado: janela do gravador, captura, foco, Esc, processamento em thread.

use super::etapas::Contexto;
use super::pipeline;
use super::servico::{Efeitos, EstadoGravador};
use crate::audio::captura::{Captura, Gravacao};
use crate::comandos::CofreEstado;
use crate::nucleo::{ErroApp, caminhos::Caminhos};
use crate::plataforma::{foco, teclado::EscutaEsc};
use crate::transcricao::{self, ConfigTranscricao};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use specta::Type;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Manager, PhysicalPosition, Wry};
use tauri_specta::Event;

pub const JANELA_GRAVADOR: &str = "gravador";
const LARGURA: f64 = 460.0;
const ALTURA: f64 = 64.0;
const MARGEM_INFERIOR: f64 = 96.0;

/// Estado do gravador, enviado a cada mudança.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct DitadoEstado(pub EstadoGravador);

/// Nível do áudio (0 a 1) enquanto grava.
#[derive(Debug, Clone, Serialize, Deserialize, Type, Event)]
pub struct DitadoNivel(pub f32);

pub struct EfeitosTauri {
    app: AppHandle<Wry>,
    inicio: Instant,
    captura: Mutex<Option<Captura>>,
    escuta: Mutex<Option<EscutaEsc>>,
    geracao: AtomicU64,
    /// Ligado depois da montagem: o que fazer quando Esc é pressionado.
    pub ao_esc: OnceLock<Arc<dyn Fn() + Send + Sync>>,
}

impl EfeitosTauri {
    pub fn novo(app: AppHandle<Wry>) -> Arc<Self> {
        Arc::new(Self {
            app,
            inicio: Instant::now(),
            captura: Mutex::new(None),
            escuta: Mutex::new(None),
            geracao: AtomicU64::new(0),
            ao_esc: OnceLock::new(),
        })
    }

    fn janela(&self) -> Option<tauri::WebviewWindow<Wry>> {
        self.app.get_webview_window(JANELA_GRAVADOR)
    }

    /// Modo ativo e a transcrição configurada nele (padrão se não há modo).
    fn config_do_modo_ativo(&self) -> Result<(Option<String>, ConfigTranscricao), ErroApp> {
        let banco = self.app.state::<Mutex<Connection>>();
        let c = banco
            .lock()
            .map_err(|_| ErroApp::interno("trava do banco"))?;
        let modo = transcricao::config::modo_ativo(&c)?;
        let config = transcricao::config::obter(&c, modo.as_deref())?;
        Ok((modo, config))
    }

    /// Guarda o áudio e o item "falhou" no histórico (RF-02.3).
    fn registrar_falha(&self, contexto: &Contexto, erro: &ErroApp) {
        let caminhos = self.app.state::<Caminhos>();
        let banco = self.app.state::<Mutex<Connection>>();
        let Ok(c) = banco.lock() else { return };
        let falha = transcricao::falha::Falha {
            modo_id: contexto.modo_id.as_deref(),
            programa: contexto.programa_em_foco.as_deref(),
            config: &contexto.config_transcricao,
            gravacao: contexto.gravacao.as_ref(),
            erro,
        };
        if let Err(e) = transcricao::falha::registrar(&c, &caminhos, &falha) {
            tracing::error!(evento = "falha_nao_registrada", codigo = %e.codigo);
        }
    }

    fn posicionar(&self, janela: &tauri::WebviewWindow<Wry>) {
        let Ok(Some(monitor)) = janela.primary_monitor() else {
            return;
        };
        let escala = monitor.scale_factor();
        let tela = monitor.size();
        let origem = monitor.position();
        let largura = (LARGURA * escala) as i32;
        let x = origem.x + (i32::try_from(tela.width).unwrap_or(0) - largura) / 2;
        let y = origem.y + i32::try_from(tela.height).unwrap_or(0)
            - ((ALTURA + MARGEM_INFERIOR) * escala) as i32;
        let _ = janela.set_position(PhysicalPosition::new(x, y));
    }
}

impl Efeitos for Arc<EfeitosTauri> {
    fn agora_ms(&self) -> u64 {
        self.inicio.elapsed().as_millis() as u64
    }

    fn mostrar_gravador(&self) {
        self.geracao.fetch_add(1, Ordering::SeqCst);
        if let Some(j) = self.janela() {
            self.posicionar(&j);
            // Janela criada com `focusable(false)`: aparece sem ativar nem tomar o foco.
            let _ = j.show();
        }
    }

    fn ocultar_gravador(&self, depois_ms: u64) {
        let geracao = self.geracao.load(Ordering::SeqCst);
        let este = Arc::clone(self);
        let esconder = move || {
            // Um ditado novo pode ter reaberto o gravador nesse intervalo.
            if este.geracao.load(Ordering::SeqCst) == geracao
                && let Some(j) = este.janela()
            {
                let _ = j.hide();
            }
        };
        if depois_ms == 0 {
            esconder();
        } else {
            std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(depois_ms));
                esconder();
            });
        }
    }

    fn emitir_estado(&self, estado: &EstadoGravador) {
        let _ = DitadoEstado(estado.clone()).emit(&self.app);
    }

    fn programa_em_foco(&self) -> Option<String> {
        foco::programa_em_foco()
    }

    fn iniciar_captura(&self) -> Result<(), ErroApp> {
        let app = self.app.clone();
        let ultimo = Mutex::new((Instant::now(), 0.0_f32));
        let captura = Captura::iniciar(move |nivel| {
            let Ok(mut u) = ultimo.lock() else { return };
            u.1 = u.1.max(nivel);
            if u.0.elapsed() >= Duration::from_millis(50) {
                let _ = DitadoNivel(u.1).emit(&app);
                *u = (Instant::now(), 0.0);
            }
        })?;
        *self
            .captura
            .lock()
            .map_err(|_| ErroApp::interno("trava da captura"))? = Some(captura);
        Ok(())
    }

    fn parar_captura(&self) -> Option<Gravacao> {
        self.captura.lock().ok()?.take().map(Captura::parar)
    }

    fn descartar_captura(&self) {
        if let Some(c) = self.captura.lock().ok().and_then(|mut c| c.take()) {
            c.descartar();
        }
    }

    fn pico_da_captura(&self) -> Option<f32> {
        self.captura.lock().ok()?.as_ref().map(Captura::pico)
    }

    fn escutar_esc(&self, ligar: bool) {
        let Ok(mut escuta) = self.escuta.lock() else {
            return;
        };
        if !ligar {
            *escuta = None;
            return;
        }
        let Some(ao_esc) = self.ao_esc.get().cloned() else {
            return;
        };
        match EscutaEsc::iniciar(move || ao_esc()) {
            Ok(e) => *escuta = Some(e),
            Err(e) => tracing::warn!(evento = "esc_indisponivel", detalhe = %e),
        }
    }

    fn conferir_transcricao(&self) -> Result<(), ErroApp> {
        let config = self.config_do_modo_ativo()?.1;
        let cofre = self.app.state::<CofreEstado>();
        crate::transcricao::exigir_chave(cofre.as_ref(), &config)
    }

    fn abrir_chaves(&self) {
        crate::bandeja::abrir_principal(&self.app, Some("/config/provedores"));
    }

    fn processar(
        &self,
        mut contexto: Contexto,
        gravacao: Option<Gravacao>,
        ao_fim: Box<dyn FnOnce(Result<Contexto, ErroApp>) + Send>,
    ) {
        let este = Arc::clone(self);
        std::thread::spawn(move || {
            let (modo_id, config) = este.config_do_modo_ativo().unwrap_or_default();
            let cofre: CofreEstado = Arc::clone(&este.app.state::<CofreEstado>());
            contexto.modo_id = modo_id;
            contexto.config_transcricao = config;
            contexto.gravacao = gravacao;
            contexto.fabrica_transcritor = Some(crate::transcricao::fabrica_com_cofre(cofre));
            let resultado = pipeline::executar(&mut contexto).map(|_| ());
            if let Err(erro) = &resultado
                && contexto.passou_por.last() == Some(&"transcrever")
            {
                este.registrar_falha(&contexto, erro);
            }
            ao_fim(resultado.map(|()| contexto));
        });
    }
}

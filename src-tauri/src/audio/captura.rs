//! Captura do microfone com cpal. O fluxo vive numa thread própria (o `Stream` não é `Send` em
//! todas as plataformas); amostras são guardadas em mono i16 na taxa do dispositivo.

use super::nivel;
use crate::nucleo::ErroApp;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread::JoinHandle;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Gravacao {
    pub amostras: Vec<i16>,
    pub taxa_hz: u32,
}

#[derive(Default)]
struct Dados {
    amostras: Vec<i16>,
    pico: f32,
}

pub struct Captura {
    parar: Arc<AtomicBool>,
    dados: Arc<Mutex<Dados>>,
    taxa_hz: u32,
    thread: Option<JoinHandle<()>>,
}

pub fn erro_sem_microfone() -> ErroApp {
    ErroApp::novo(
        "sem_acesso_microfone",
        "Sem acesso ao microfone. Conceda a permissão em Configurações › Permissões.",
    )
}

impl Captura {
    /// Abre o microfone padrão. `ao_nivel` recebe o nível (0..1) de cada bloco capturado.
    pub fn iniciar(ao_nivel: impl Fn(f32) + Send + 'static) -> Result<Self, ErroApp> {
        let parar = Arc::new(AtomicBool::new(false));
        let dados = Arc::new(Mutex::new(Dados::default()));
        let (tx, rx) = mpsc::sync_channel::<Result<u32, ErroApp>>(1);
        let (p, d) = (Arc::clone(&parar), Arc::clone(&dados));
        let thread = std::thread::spawn(move || match abrir(&d, ao_nivel) {
            Ok((stream, taxa)) => {
                let _ = tx.send(Ok(taxa));
                while !p.load(Ordering::Relaxed) {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                }
                drop(stream);
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        });
        match rx.recv() {
            Ok(Ok(taxa_hz)) => Ok(Self {
                parar,
                dados,
                taxa_hz,
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => Err(erro_sem_microfone()),
        }
    }

    /// Maior amplitude já capturada (0,0 = silêncio digital: típico de microfone sem permissão).
    pub fn pico(&self) -> f32 {
        self.dados.lock().map(|d| d.pico).unwrap_or(0.0)
    }

    pub fn parar(mut self) -> Gravacao {
        self.encerrar();
        let amostras = self
            .dados
            .lock()
            .map(|mut d| std::mem::take(&mut d.amostras))
            .unwrap_or_default();
        Gravacao {
            amostras,
            taxa_hz: self.taxa_hz,
        }
    }

    /// Descarta o áudio capturado.
    pub fn descartar(mut self) {
        self.encerrar();
        if let Ok(mut d) = self.dados.lock() {
            d.amostras.clear();
        }
    }

    fn encerrar(&mut self) {
        self.parar.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

impl Drop for Captura {
    fn drop(&mut self) {
        self.encerrar();
    }
}

fn abrir(
    dados: &Arc<Mutex<Dados>>,
    ao_nivel: impl Fn(f32) + Send + 'static,
) -> Result<(cpal::Stream, u32), ErroApp> {
    let dispositivo = cpal::default_host()
        .default_input_device()
        .ok_or_else(erro_sem_microfone)?;
    let config = dispositivo.default_input_config().map_err(|e| {
        tracing::warn!(evento = "microfone_sem_configuracao", detalhe = %e);
        erro_sem_microfone()
    })?;
    let canais = usize::from(config.channels()).max(1);
    let taxa = config.sample_rate();
    let formato = config.sample_format();
    let config: cpal::StreamConfig = config.into();
    let d = Arc::clone(dados);
    let processa = move |mono: Vec<f32>| {
        ao_nivel(nivel::nivel(&mono));
        if let Ok(mut d) = d.lock() {
            d.pico = d.pico.max(nivel::pico(&mono));
            d.amostras.extend(mono.iter().map(|a| nivel::para_i16(*a)));
        }
    };
    let aviso_erro = |e| tracing::warn!(evento = "erro_no_fluxo_de_audio", detalhe = %e);
    let stream = match formato {
        cpal::SampleFormat::F32 => dispositivo.build_input_stream(
            config,
            move |bloco: &[f32], _| processa(para_mono(bloco, canais, |a| a)),
            aviso_erro,
            None,
        ),
        cpal::SampleFormat::I16 => dispositivo.build_input_stream(
            config,
            move |bloco: &[i16], _| processa(para_mono(bloco, canais, |a| f32::from(a) / 32768.0)),
            aviso_erro,
            None,
        ),
        cpal::SampleFormat::U16 => dispositivo.build_input_stream(
            config,
            move |bloco: &[u16], _| {
                processa(para_mono(bloco, canais, |a| {
                    (f32::from(a) - 32768.0) / 32768.0
                }));
            },
            aviso_erro,
            None,
        ),
        outro => {
            tracing::warn!(evento = "formato_de_audio_nao_suportado", formato = ?outro);
            return Err(erro_sem_microfone());
        }
    }
    .map_err(|e| {
        tracing::warn!(evento = "microfone_nao_abriu", detalhe = %e);
        erro_sem_microfone()
    })?;
    stream.play().map_err(|e| {
        tracing::warn!(evento = "microfone_nao_iniciou", detalhe = %e);
        erro_sem_microfone()
    })?;
    Ok((stream, taxa))
}

/// Média dos canais de cada quadro.
pub fn para_mono<T: Copy>(bloco: &[T], canais: usize, para_f32: impl Fn(T) -> f32) -> Vec<f32> {
    bloco
        .chunks(canais)
        .map(|q| q.iter().map(|a| para_f32(*a)).sum::<f32>() / q.len() as f32)
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn mono_tira_a_media_dos_canais() {
        let estereo = [1.0_f32, 0.0, 0.5, 0.5, -1.0, 1.0];
        assert_eq!(para_mono(&estereo, 2, |a| a), vec![0.5, 0.5, 0.0]);
    }

    #[test]
    fn mono_com_um_canal_nao_altera() {
        assert_eq!(para_mono(&[0.1_f32, 0.2], 1, |a| a), vec![0.1, 0.2]);
    }

    #[test]
    fn erro_de_microfone_traz_o_caminho_para_conceder() {
        let e = erro_sem_microfone();
        assert_eq!(e.codigo, "sem_acesso_microfone");
        assert!(e.mensagem.contains("Permissões"));
    }
}

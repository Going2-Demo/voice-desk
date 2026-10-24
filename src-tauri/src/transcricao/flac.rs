//! Converte a gravação (mono i16 na taxa do microfone) em FLAC 16 kHz mono, o formato enviado
//! ao Groq: 10 min ficam perto de 10 MB, abaixo do limite de 25 MB do plano gratuito.

use crate::audio::captura::Gravacao;
use crate::nucleo::ErroApp;
use flacenc::component::BitRepr;
use flacenc::error::Verify;

pub const TAXA_ENVIO_HZ: u32 = 16_000;

/// Reamostragem linear; suficiente para fala e sem dependência extra.
pub fn reamostrar(amostras: &[i16], de_hz: u32, para_hz: u32) -> Vec<i16> {
    if de_hz == para_hz || amostras.is_empty() || de_hz == 0 {
        return amostras.to_vec();
    }
    let saida = (amostras.len() as u64 * u64::from(para_hz) / u64::from(de_hz)) as usize;
    let passo = f64::from(de_hz) / f64::from(para_hz);
    (0..saida)
        .map(|i| {
            let pos = i as f64 * passo;
            let a = pos.floor() as usize;
            let b = (a + 1).min(amostras.len() - 1);
            let frac = pos - a as f64;
            (f64::from(amostras[a]) * (1.0 - frac) + f64::from(amostras[b]) * frac).round() as i16
        })
        .collect()
}

pub fn codificar(gravacao: &Gravacao) -> Result<Vec<u8>, ErroApp> {
    if gravacao.amostras.is_empty() || gravacao.taxa_hz == 0 {
        return Err(ErroApp::novo("sem_audio", "Não há áudio para transcrever."));
    }
    let pcm = reamostrar(&gravacao.amostras, gravacao.taxa_hz, TAXA_ENVIO_HZ);
    let amostras: Vec<i32> = pcm.iter().map(|a| i32::from(*a)).collect();
    let config = flacenc::config::Encoder::default()
        .into_verified()
        .map_err(|(_, e)| ErroApp::interno(format!("config FLAC: {e:?}")))?;
    let fonte = flacenc::source::MemSource::from_samples(&amostras, 1, 16, TAXA_ENVIO_HZ as usize);
    let fluxo = flacenc::encode_with_fixed_block_size(&config, fonte, config.block_size)
        .map_err(|e| ErroApp::interno(format!("codificar FLAC: {e:?}")))?;
    let mut saida = flacenc::bitsink::ByteSink::new();
    fluxo
        .write(&mut saida)
        .map_err(|e| ErroApp::interno(format!("escrever FLAC: {e:?}")))?;
    Ok(saida.as_slice().to_vec())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tom(taxa_hz: u32, segundos: u32) -> Gravacao {
        let n = taxa_hz * segundos;
        Gravacao {
            taxa_hz,
            amostras: (0..n)
                .map(|i| {
                    ((i as f32 * 440.0 * std::f32::consts::TAU / taxa_hz as f32).sin() * 8000.0)
                        as i16
                })
                .collect(),
        }
    }

    #[test]
    fn gera_flac_valido() {
        let bytes = codificar(&tom(48_000, 1)).unwrap();
        assert_eq!(&bytes[..4], b"fLaC");
        assert!(bytes.len() < 48_000 * 2, "deve comprimir");
    }

    #[test]
    fn reamostra_para_16khz() {
        assert_eq!(reamostrar(&vec![0; 48_000], 48_000, 16_000).len(), 16_000);
        assert_eq!(reamostrar(&[1, 2, 3], 16_000, 16_000), [1, 2, 3]);
    }

    #[test]
    fn sem_audio_e_erro() {
        assert_eq!(
            codificar(&Gravacao::default()).unwrap_err().codigo,
            "sem_audio"
        );
    }
}

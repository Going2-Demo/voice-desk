//! Nível do áudio para o medidor do gravador: valor de 0,0 a 1,0 por bloco de amostras.

/// Nível suavizado do bloco (RMS em dBFS mapeado de -60 dB a 0 dB para 0..1).
pub fn nivel(amostras: &[f32]) -> f32 {
    if amostras.is_empty() {
        return 0.0;
    }
    let soma: f32 = amostras.iter().map(|a| a * a).sum();
    let rms = (soma / amostras.len() as f32).sqrt();
    if rms <= 0.0 {
        return 0.0;
    }
    let db = 20.0 * rms.log10();
    ((db + 60.0) / 60.0).clamp(0.0, 1.0)
}

/// Maior amplitude absoluta do bloco.
pub fn pico(amostras: &[f32]) -> f32 {
    amostras.iter().fold(0.0_f32, |m, a| m.max(a.abs()))
}

/// Converte amostras f32 (-1..1) em i16.
pub fn para_i16(a: f32) -> i16 {
    (a.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn silencio_tem_nivel_zero() {
        assert_eq!(nivel(&[0.0; 512]), 0.0);
        assert_eq!(nivel(&[]), 0.0);
    }

    #[test]
    fn sinal_forte_tem_nivel_perto_de_um() {
        assert!(nivel(&[0.9, -0.9, 0.9, -0.9]) > 0.9);
        assert_eq!(nivel(&[1.0; 8]), 1.0);
    }

    #[test]
    fn nivel_cresce_com_a_amplitude() {
        let baixo = nivel(&[0.01; 64]);
        let medio = nivel(&[0.1; 64]);
        let alto = nivel(&[0.5; 64]);
        assert!(baixo < medio && medio < alto);
        assert!((0.0..=1.0).contains(&baixo));
    }

    #[test]
    fn pico_e_conversao_respeitam_os_limites() {
        assert_eq!(pico(&[0.1, -0.7, 0.3]), 0.7);
        assert_eq!(para_i16(2.0), i16::MAX);
        assert_eq!(para_i16(-2.0), -i16::MAX);
        assert_eq!(para_i16(0.0), 0);
    }
}

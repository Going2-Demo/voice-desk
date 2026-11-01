//! Etapa `descartar_sem_fala` (F-26): com o descarte ligado, uma gravação sem fala encerra o
//! ditado sem chamar o provedor, sem colar e sem item no histórico (RF-26.2).

use super::{Contexto, Fluxo};
use crate::audio::fala;
use crate::nucleo::ErroApp;

pub fn executar(contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    if !contexto.descartar_sem_fala {
        return Ok(Fluxo::Seguir);
    }
    let Some(gravacao) = contexto.gravacao.as_ref() else {
        return Ok(Fluxo::Seguir);
    };
    if fala::tem_fala(gravacao) {
        return Ok(Fluxo::Seguir);
    }
    tracing::info!(
        evento = "ditado_sem_fala",
        amostras = gravacao.amostras.len()
    );
    contexto.sem_fala = true;
    contexto.gravacao = None;
    Ok(Fluxo::Parar)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::audio::captura::Gravacao;

    fn contexto(amostras: Vec<i16>, descartar: bool) -> Contexto {
        Contexto {
            gravacao: Some(Gravacao {
                amostras,
                taxa_hz: 16_000,
            }),
            descartar_sem_fala: descartar,
            ..Contexto::default()
        }
    }

    #[test]
    fn silencio_com_descarte_ligado_para_e_marca_sem_fala() {
        let mut c = contexto(vec![0; 80_000], true);
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Parar);
        assert!(c.sem_fala);
        assert!(c.gravacao.is_none());
    }

    #[test]
    fn silencio_com_descarte_desligado_segue_para_o_provedor() {
        let mut c = contexto(vec![0; 80_000], false);
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
        assert!(!c.sem_fala);
        assert!(c.gravacao.is_some());
    }

    #[test]
    fn fala_audivel_segue() {
        let amostras = (0..16_000)
            .map(|i| if i % 2 == 0 { 300 } else { -300 })
            .collect();
        let mut c = contexto(amostras, true);
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
        assert!(!c.sem_fala);
    }

    #[test]
    fn sem_gravacao_segue() {
        let mut c = Contexto {
            descartar_sem_fala: true,
            ..Contexto::default()
        };
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
    }
}

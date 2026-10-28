//! Etapa `substituir` (F-08): aplica as substituições ativas ao texto transcrito, antes da
//! limpeza e do refinamento (RF-08.3). Sem regras, o texto segue como está.

use super::{Contexto, Fluxo};
use crate::nucleo::ErroApp;
use crate::substituicoes;

pub fn executar(contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    if !contexto.substituicoes.is_empty() {
        contexto.texto_final =
            substituicoes::aplicar(&contexto.substituicoes, &contexto.texto_final);
    }
    Ok(Fluxo::Seguir)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::substituicoes::Regra;

    #[test]
    fn troca_no_texto_final_e_preserva_o_transcrito() {
        let mut c = Contexto {
            texto_transcrito: "manda um pix".into(),
            texto_final: "manda um pix".into(),
            substituicoes: vec![Regra::nova("pix", "PIX")],
            ..Contexto::default()
        };
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
        assert_eq!(c.texto_final, "manda um PIX");
        assert_eq!(c.texto_transcrito, "manda um pix");
    }

    #[test]
    fn sem_regras_nada_muda() {
        let mut c = Contexto {
            texto_final: "olá".into(),
            ..Contexto::default()
        };
        executar(&mut c).unwrap();
        assert_eq!(c.texto_final, "olá");
    }
}

//! Etapa `limpar` (F-09): tira as palavras de preenchimento e formata o texto, sem IA, conforme o
//! que o modo liga. Com tudo desligado, o texto segue como veio da transcrição.

use super::{Contexto, Fluxo};
use crate::limpeza::algoritmo;
use crate::nucleo::ErroApp;

pub fn executar(contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    let config = &contexto.config_limpeza;
    if config.remover_preenchimento || config.formatar {
        contexto.texto_final = algoritmo::limpar(
            &contexto.texto_final,
            &config.palavras,
            config.remover_preenchimento,
            config.formatar,
        );
    }
    Ok(Fluxo::Seguir)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::limpeza::ConfigLimpeza;

    fn contexto(texto: &str, remover: bool, formatar: bool) -> Contexto {
        Contexto {
            texto_transcrito: texto.into(),
            texto_final: texto.into(),
            config_limpeza: ConfigLimpeza {
                remover_preenchimento: remover,
                formatar,
                palavras: vec!["hã…".into(), "tipo".into()],
            },
            ..Contexto::default()
        }
    }

    #[test]
    fn modo_chat_com_tudo_ligado_cola_o_texto_limpo() {
        let mut c = contexto("hã… chego em dez minutos", true, true);
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
        assert_eq!(c.texto_final, "Chego em dez minutos.");
        assert_eq!(c.texto_transcrito, "hã… chego em dez minutos");
    }

    #[test]
    fn ditado_literal_com_tudo_desligado_nao_altera_nada() {
        let mut c = contexto("hã… chego em dez minutos", false, false);
        executar(&mut c).unwrap();
        assert_eq!(c.texto_final, "hã… chego em dez minutos");
    }

    #[test]
    fn so_formatar_e_so_remover_agem_separados() {
        let mut c = contexto("olá  tudo bem", false, true);
        executar(&mut c).unwrap();
        assert_eq!(c.texto_final, "Olá tudo bem.");
        let mut c = contexto("é tipo assim", true, false);
        executar(&mut c).unwrap();
        assert_eq!(c.texto_final, "é assim");
    }

    #[test]
    fn contexto_sem_configuracao_nao_mexe_no_texto() {
        let mut c = Contexto {
            texto_final: "tipo assim".into(),
            ..Contexto::default()
        };
        executar(&mut c).unwrap();
        assert_eq!(c.texto_final, "tipo assim");
    }
}

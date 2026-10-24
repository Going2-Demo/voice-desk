//! Etapa `colar` (F-03): cola `texto_final` no programa em foco e registra o resultado.

use super::{Contexto, Fluxo};
use crate::ditado::colagem::{self, Sistema, SistemaReal};
use crate::nucleo::ErroApp;

pub fn executar(contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    executar_com(&mut SistemaReal, contexto)
}

fn executar_com(sistema: &mut dyn Sistema, contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    // Sem texto não há o que colar: não mexe na área de transferência do usuário.
    if contexto.texto_final.is_empty() {
        return Ok(Fluxo::Seguir);
    }
    let resultado = colagem::colar(sistema, &contexto.texto_final, contexto.enviar_enter);
    tracing::info!(
        evento = "colagem_resultado",
        resultado = resultado.estado_historico()
    );
    contexto.colagem = Some(resultado);
    Ok(Fluxo::Seguir)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::ditado::colagem::ResultadoColagem;
    use crate::plataforma::area_transferencia::Conteudo;
    use std::time::Duration;

    #[derive(Default)]
    struct Falso {
        chamadas: usize,
        colar_falha: bool,
        enters: usize,
    }

    impl Sistema for Falso {
        fn ler_area(&mut self) -> Conteudo {
            self.chamadas += 1;
            Conteudo::Vazio
        }
        fn escrever_texto(&mut self, _: &str) -> Result<(), String> {
            Ok(())
        }
        fn restaurar_area(&mut self, _: &Conteudo) -> Result<(), String> {
            Ok(())
        }
        fn simular_colar(&mut self) -> Result<(), String> {
            if self.colar_falha {
                Err("negado".into())
            } else {
                Ok(())
            }
        }
        fn simular_enter(&mut self) -> Result<(), String> {
            self.enters += 1;
            Ok(())
        }
        fn esperar(&mut self, _: Duration) {}
    }

    fn com_texto(texto: &str) -> Contexto {
        Contexto {
            texto_final: texto.into(),
            ..Contexto::default()
        }
    }

    #[test]
    fn sem_texto_final_nao_toca_na_area_de_transferencia() {
        let mut s = Falso::default();
        let mut c = Contexto::default();
        assert_eq!(executar_com(&mut s, &mut c).unwrap(), Fluxo::Seguir);
        assert_eq!(s.chamadas, 0);
        assert_eq!(c.colagem, None);
    }

    #[test]
    fn registra_colado() {
        let mut c = com_texto("oi");
        executar_com(&mut Falso::default(), &mut c).unwrap();
        assert_eq!(c.colagem, Some(ResultadoColagem::Colado));
    }

    #[test]
    fn registra_nao_colado_e_segue_sem_erro() {
        let mut c = com_texto("oi");
        let mut s = Falso {
            colar_falha: true,
            ..Falso::default()
        };
        assert_eq!(executar_com(&mut s, &mut c).unwrap(), Fluxo::Seguir);
        assert_eq!(c.colagem, Some(ResultadoColagem::NaoColado));
    }

    #[test]
    fn enter_do_modo_chega_na_colagem() {
        let mut s = Falso::default();
        let mut c = Contexto {
            enviar_enter: true,
            ..com_texto("oi")
        };
        executar_com(&mut s, &mut c).unwrap();
        assert_eq!(s.enters, 1);
        let mut c = com_texto("oi");
        executar_com(&mut s, &mut c).unwrap();
        assert_eq!(s.enters, 1);
    }
}

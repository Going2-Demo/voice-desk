//! Colagem do texto final no programa em foco (F-03, ADR-07): guarda a área de transferência,
//! escreve o texto, simula Cmd/Ctrl+V, devolve o conteúdo anterior e, se o modo pedir, envia Enter.
//! O sistema entra por `Sistema`, então a ordem e os desvios de erro são testáveis sem teclado real.

use crate::plataforma::area_transferencia::{self, Conteudo};
use crate::plataforma::colagem;
use std::time::Duration;

/// Espera entre a colagem e a devolução da área de transferência (ajustável por medição).
pub const ESPERA_ANTES_DE_DEVOLVER: Duration = Duration::from_millis(150);

/// Resultado registrado no histórico (RF-03.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResultadoColagem {
    Colado,
    NaoColado,
    Cancelado,
}

impl ResultadoColagem {
    /// Valor da coluna `historico_item.estado`.
    pub const fn estado_historico(self) -> &'static str {
        match self {
            Self::Colado => "colado",
            Self::NaoColado => "nao_colado",
            Self::Cancelado => "cancelado",
        }
    }
}

pub trait Sistema {
    fn ler_area(&mut self) -> Conteudo;
    fn escrever_texto(&mut self, texto: &str) -> Result<(), String>;
    fn restaurar_area(&mut self, conteudo: &Conteudo) -> Result<(), String>;
    fn simular_colar(&mut self) -> Result<(), String>;
    fn simular_enter(&mut self) -> Result<(), String>;
    fn esperar(&mut self, tempo: Duration);
}

/// Cola `texto` no programa em foco. Em `NaoColado` o texto fica na área de transferência para
/// o usuário colar à mão (RF-03.4). Nunca registra o texto em log.
pub fn colar(sistema: &mut dyn Sistema, texto: &str, enviar_enter: bool) -> ResultadoColagem {
    let anterior = sistema.ler_area();
    if let Err(e) = sistema.escrever_texto(texto) {
        tracing::warn!(evento = "colagem_falhou", etapa = "escrever", detalhe = %e);
        return ResultadoColagem::NaoColado;
    }
    if let Err(e) = sistema.simular_colar() {
        tracing::warn!(evento = "colagem_falhou", etapa = "simular", detalhe = %e);
        return ResultadoColagem::NaoColado;
    }
    sistema.esperar(ESPERA_ANTES_DE_DEVOLVER);
    if let Err(e) = sistema.restaurar_area(&anterior) {
        tracing::warn!(evento = "colagem_falhou", etapa = "restaurar", detalhe = %e);
    }
    if enviar_enter && let Err(e) = sistema.simular_enter() {
        tracing::warn!(evento = "colagem_falhou", etapa = "enter", detalhe = %e);
    }
    ResultadoColagem::Colado
}

/// Sistema real: arboard (área de transferência) e enigo (teclas), via `plataforma/`.
pub struct SistemaReal;

impl Sistema for SistemaReal {
    fn ler_area(&mut self) -> Conteudo {
        area_transferencia::ler()
    }
    fn escrever_texto(&mut self, texto: &str) -> Result<(), String> {
        area_transferencia::escrever_texto(texto)
    }
    fn restaurar_area(&mut self, conteudo: &Conteudo) -> Result<(), String> {
        area_transferencia::restaurar(conteudo)
    }
    fn simular_colar(&mut self) -> Result<(), String> {
        colagem::simular_colar()
    }
    fn simular_enter(&mut self) -> Result<(), String> {
        colagem::simular_enter()
    }
    fn esperar(&mut self, tempo: Duration) {
        std::thread::sleep(tempo);
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[derive(Default)]
    struct Falso {
        area: Option<Conteudo>,
        passos: Vec<String>,
        falha_escrever: bool,
        falha_colar: bool,
        falha_restaurar: bool,
        falha_enter: bool,
    }

    impl Falso {
        fn com_area(texto: &str) -> Self {
            Self {
                area: Some(Conteudo::Texto(texto.into())),
                ..Self::default()
            }
        }
        fn area(&self) -> Conteudo {
            self.area.clone().unwrap_or(Conteudo::Vazio)
        }
    }

    impl Sistema for Falso {
        fn ler_area(&mut self) -> Conteudo {
            self.passos.push("ler".into());
            self.area()
        }
        fn escrever_texto(&mut self, texto: &str) -> Result<(), String> {
            self.passos.push(format!("escrever:{texto}"));
            if self.falha_escrever {
                return Err("sem acesso".into());
            }
            self.area = Some(Conteudo::Texto(texto.into()));
            Ok(())
        }
        fn restaurar_area(&mut self, conteudo: &Conteudo) -> Result<(), String> {
            self.passos.push("restaurar".into());
            if self.falha_restaurar {
                return Err("sem acesso".into());
            }
            self.area = Some(conteudo.clone());
            Ok(())
        }
        fn simular_colar(&mut self) -> Result<(), String> {
            self.passos.push("colar".into());
            if self.falha_colar {
                Err("acessibilidade negada".into())
            } else {
                Ok(())
            }
        }
        fn simular_enter(&mut self) -> Result<(), String> {
            self.passos.push("enter".into());
            if self.falha_enter {
                Err("x".into())
            } else {
                Ok(())
            }
        }
        fn esperar(&mut self, tempo: Duration) {
            self.passos.push(format!("esperar:{}", tempo.as_millis()));
        }
    }

    #[test]
    fn cola_espera_e_devolve_a_area_anterior() {
        let mut s = Falso::com_area("abc");
        let r = colar(&mut s, "olá 😀\nlinha 2", false);
        assert_eq!(r, ResultadoColagem::Colado);
        assert_eq!(
            s.passos,
            [
                "ler",
                "escrever:olá 😀\nlinha 2",
                "colar",
                "esperar:150",
                "restaurar"
            ]
        );
        assert_eq!(s.area(), Conteudo::Texto("abc".into()));
    }

    #[test]
    fn devolve_imagem_e_area_vazia_tambem() {
        let img = Conteudo::Imagem {
            largura: 1,
            altura: 1,
            rgba: vec![1, 2, 3, 4],
        };
        let mut s = Falso {
            area: Some(img.clone()),
            ..Falso::default()
        };
        colar(&mut s, "x", false);
        assert_eq!(s.area(), img);
        let mut s = Falso::default();
        colar(&mut s, "x", false);
        assert_eq!(s.area(), Conteudo::Vazio);
    }

    #[test]
    fn com_enter_ligado_envia_enter_depois_de_devolver_a_area() {
        let mut s = Falso::com_area("abc");
        assert_eq!(colar(&mut s, "oi", true), ResultadoColagem::Colado);
        assert_eq!(&s.passos[s.passos.len() - 2..], ["restaurar", "enter"]);
    }

    #[test]
    fn com_enter_desligado_nao_envia_enter() {
        let mut s = Falso::com_area("abc");
        colar(&mut s, "oi", false);
        assert!(!s.passos.contains(&"enter".to_string()));
    }

    #[test]
    fn sem_permissao_o_texto_fica_na_area_e_nao_restaura_nem_envia_enter() {
        let mut s = Falso {
            falha_colar: true,
            ..Falso::com_area("abc")
        };
        assert_eq!(colar(&mut s, "ditado", true), ResultadoColagem::NaoColado);
        assert_eq!(s.area(), Conteudo::Texto("ditado".into()));
        assert!(!s.passos.contains(&"restaurar".to_string()));
        assert!(!s.passos.contains(&"enter".to_string()));
    }

    #[test]
    fn falha_ao_escrever_na_area_e_nao_colado_sem_simular_teclas() {
        let mut s = Falso {
            falha_escrever: true,
            ..Falso::com_area("abc")
        };
        assert_eq!(colar(&mut s, "x", true), ResultadoColagem::NaoColado);
        assert!(!s.passos.contains(&"colar".to_string()));
    }

    #[test]
    fn falha_ao_restaurar_ou_no_enter_nao_desfaz_a_colagem() {
        let mut s = Falso {
            falha_restaurar: true,
            falha_enter: true,
            ..Falso::com_area("abc")
        };
        assert_eq!(colar(&mut s, "x", true), ResultadoColagem::Colado);
    }

    #[test]
    fn estados_do_historico() {
        assert_eq!(ResultadoColagem::Colado.estado_historico(), "colado");
        assert_eq!(ResultadoColagem::NaoColado.estado_historico(), "nao_colado");
        assert_eq!(ResultadoColagem::Cancelado.estado_historico(), "cancelado");
    }
}

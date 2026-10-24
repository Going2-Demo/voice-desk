//! "Iniciar com o sistema" (RF-05.2): preferência local, padrão ligado, aplicada no sistema por
//! uma `InicioAutomatico` (o plugin `autostart` em produção, um falso nos testes).

use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension};

pub const CHAVE: &str = "bandeja.iniciar_com_o_sistema";

pub trait InicioAutomatico {
    fn ligado(&self) -> Result<bool, ErroApp>;
    fn definir(&self, ligado: bool) -> Result<(), ErroApp>;
}

/// Valor guardado; sem registro, o padrão é ligado.
pub fn ler(conexao: &Connection) -> Result<bool, ErroApp> {
    let valor: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = ?1",
            [CHAVE],
            |l| l.get(0),
        )
        .optional()?;
    Ok(valor.as_deref().map(|v| v != "false").unwrap_or(true))
}

fn gravar(conexao: &Connection, ligado: bool) -> Result<(), ErroApp> {
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        rusqlite::params![CHAVE, if ligado { "true" } else { "false" }],
    )?;
    Ok(())
}

/// Liga ou desliga no sistema e só então guarda a preferência; se o sistema recusar, nada muda.
pub fn alterar(
    conexao: &Connection,
    sistema: &dyn InicioAutomatico,
    ligado: bool,
) -> Result<bool, ErroApp> {
    sistema.definir(ligado).map_err(|e| {
        tracing::warn!(evento = "inicio_automatico_falhou", detalhe = %e);
        ErroApp::novo(
            "inicio_automatico_indisponivel",
            "Não foi possível mudar \"iniciar com o sistema\". Tente de novo; se repetir, veja os logs em Sobre.",
        )
    })?;
    gravar(conexao, ligado)?;
    Ok(ligado)
}

/// Na abertura, o sistema passa a refletir a preferência (padrão ligado na primeira execução).
pub fn reconciliar(conexao: &Connection, sistema: &dyn InicioAutomatico) -> Result<bool, ErroApp> {
    let desejado = ler(conexao)?;
    if sistema.ligado()? != desejado {
        sistema.definir(desejado)?;
    }
    Ok(desejado)
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco;
    use std::cell::Cell;

    struct Falso {
        ligado: Cell<bool>,
        falha: bool,
    }
    impl Falso {
        fn novo(ligado: bool) -> Self {
            Self {
                ligado: Cell::new(ligado),
                falha: false,
            }
        }
    }
    impl InicioAutomatico for Falso {
        fn ligado(&self) -> Result<bool, ErroApp> {
            Ok(self.ligado.get())
        }
        fn definir(&self, ligado: bool) -> Result<(), ErroApp> {
            if self.falha {
                return Err(ErroApp::novo("x", "falhou"));
            }
            self.ligado.set(ligado);
            Ok(())
        }
    }

    fn base() -> Connection {
        let mut c = Connection::open_in_memory().unwrap();
        banco::migrar(&mut c, banco::MIGRACOES).unwrap();
        c
    }

    #[test]
    fn padrao_e_ligado() {
        assert!(ler(&base()).unwrap());
    }

    #[test]
    fn primeira_abertura_liga_no_sistema() {
        let c = base();
        let s = Falso::novo(false);
        assert!(reconciliar(&c, &s).unwrap());
        assert!(s.ligado.get());
    }

    #[test]
    fn desligar_persiste_e_desliga_no_sistema() {
        let c = base();
        let s = Falso::novo(true);
        assert!(!alterar(&c, &s, false).unwrap());
        assert!(!s.ligado.get());
        assert!(!ler(&c).unwrap());
        assert!(!reconciliar(&c, &s).unwrap());
        assert!(!s.ligado.get());
    }

    #[test]
    fn religar_volta_ao_padrao() {
        let c = base();
        let s = Falso::novo(true);
        alterar(&c, &s, false).unwrap();
        alterar(&c, &s, true).unwrap();
        assert!(ler(&c).unwrap() && s.ligado.get());
    }

    #[test]
    fn falha_do_sistema_nao_altera_a_preferencia() {
        let c = base();
        let s = Falso {
            ligado: Cell::new(true),
            falha: true,
        };
        let e = alterar(&c, &s, false).unwrap_err();
        assert_eq!(e.codigo, "inicio_automatico_indisponivel");
        assert!(ler(&c).unwrap());
    }
}

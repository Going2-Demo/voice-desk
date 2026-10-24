//! Ordem das etapas do ditado. Não muda sem passar pelo Otávio.

use super::etapas::{self, Contexto, Fluxo};
use crate::nucleo::ErroApp;

type Etapa = fn(&mut Contexto) -> Result<Fluxo, ErroApp>;

pub const ETAPAS: &[(&str, Etapa)] = &[
    ("ao_iniciar", etapas::ao_iniciar::executar),
    ("captura", etapas::captura::executar),
    ("ao_parar", etapas::ao_parar::executar),
    ("contexto", etapas::contexto::executar),
    ("descartar_sem_fala", etapas::descartar_sem_fala::executar),
    ("transcrever", etapas::transcrever::executar),
    ("gatilho", etapas::gatilho::executar),
    ("substituir", etapas::substituir::executar),
    ("limpar", etapas::limpar::executar),
    ("refinar", etapas::refinar::executar),
    ("colar", etapas::colar::executar),
    ("registrar", etapas::registrar::executar),
];

pub fn executar(contexto: &mut Contexto) -> Result<Fluxo, ErroApp> {
    for (nome, etapa) in ETAPAS {
        contexto.passou_por.push(nome);
        if etapa(contexto)? == Fluxo::Parar {
            return Ok(Fluxo::Parar);
        }
    }
    Ok(Fluxo::Seguir)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn ordem_das_etapas_e_a_da_arquitetura() {
        let nomes: Vec<_> = ETAPAS.iter().map(|(n, _)| *n).collect();
        assert_eq!(
            nomes,
            [
                "ao_iniciar",
                "captura",
                "ao_parar",
                "contexto",
                "descartar_sem_fala",
                "transcrever",
                "gatilho",
                "substituir",
                "limpar",
                "refinar",
                "colar",
                "registrar"
            ]
        );
    }

    #[test]
    fn com_etapas_provisorias_passa_por_todas_e_segue() {
        let mut c = Contexto::default();
        assert_eq!(executar(&mut c).unwrap(), Fluxo::Seguir);
        assert_eq!(c.passou_por.len(), ETAPAS.len());
    }

    #[test]
    fn etapa_que_manda_parar_interrompe_o_resto() {
        fn para(_: &mut Contexto) -> Result<Fluxo, ErroApp> {
            Ok(Fluxo::Parar)
        }
        let etapas: &[(&str, Etapa)] = &[("a", para), ("b", etapas::captura::executar)];
        let mut c = Contexto::default();
        for (n, e) in etapas {
            c.passou_por.push(n);
            if e(&mut c).unwrap() == Fluxo::Parar {
                break;
            }
        }
        assert_eq!(c.passou_por, ["a"]);
    }
}

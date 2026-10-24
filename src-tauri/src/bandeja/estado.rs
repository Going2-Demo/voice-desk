//! Estado do ícone da bandeja (RF-05.3): o que o usuário vê enquanto o app está quieto.

use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "snake_case")]
pub enum EstadoIcone {
    #[default]
    Parado,
    Gravando,
    Processando,
    Erro,
}

impl EstadoIcone {
    /// Dica de texto do ícone: o estado nunca depende só do símbolo.
    pub const fn dica(self) -> &'static str {
        match self {
            Self::Parado => "Voice Desk: parado",
            Self::Gravando => "Voice Desk: gravando",
            Self::Processando => "Voice Desk: processando",
            Self::Erro => "Voice Desk: erro, abra o menu",
        }
    }

    pub const fn png(self) -> &'static [u8] {
        match self {
            Self::Parado => include_bytes!("../../icons/bandeja/parado.png"),
            Self::Gravando => include_bytes!("../../icons/bandeja/gravando.png"),
            Self::Processando => include_bytes!("../../icons/bandeja/processando.png"),
            Self::Erro => include_bytes!("../../icons/bandeja/erro.png"),
        }
    }

    pub const fn gravando(self) -> bool {
        matches!(self, Self::Gravando)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    const TODOS: [EstadoIcone; 4] = [
        EstadoIcone::Parado,
        EstadoIcone::Gravando,
        EstadoIcone::Processando,
        EstadoIcone::Erro,
    ];

    #[test]
    fn cada_estado_tem_dica_e_aparencia_distintas() {
        let dicas: std::collections::HashSet<_> = TODOS.iter().map(|e| e.dica()).collect();
        let imagens: std::collections::HashSet<_> = TODOS.iter().map(|e| e.png()).collect();
        assert_eq!(dicas.len(), 4);
        assert_eq!(imagens.len(), 4);
    }

    #[test]
    fn imagens_sao_png_validos() {
        for e in TODOS {
            assert_eq!(&e.png()[1..4], b"PNG", "{e:?}");
        }
    }

    #[test]
    fn so_gravando_conta_como_ditando() {
        assert!(EstadoIcone::Gravando.gravando());
        assert!(!EstadoIcone::Processando.gravando());
    }

    #[test]
    fn serializa_em_snake_case() {
        assert_eq!(
            serde_json::to_string(&EstadoIcone::Processando).unwrap(),
            "\"processando\""
        );
    }
}

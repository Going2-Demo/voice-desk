//! Permissões do sistema que o ditado precisa (microfone e, no Mac, Acessibilidade). Só aqui há
//! `cfg(target_os)`; o resto do núcleo fala com `Situacao` e com as funções abaixo.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod windows;

#[cfg(target_os = "macos")]
use macos as sistema;
#[cfg(not(target_os = "macos"))]
use windows as sistema;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Situacao {
    Concedida,
    Negada,
    /// O sistema ainda não perguntou ao usuário.
    Pendente,
    /// A plataforma não tem essa permissão (Acessibilidade no Windows).
    NaoSeAplica,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permissao {
    Microfone,
    Acessibilidade,
}

pub fn situacao(permissao: Permissao) -> Situacao {
    match permissao {
        Permissao::Microfone => sistema::microfone(),
        Permissao::Acessibilidade => sistema::acessibilidade(),
    }
}

/// Valores crus do sistema para o log de diagnóstico (vazio fora do Mac).
pub fn detalhe() -> String {
    sistema::detalhe()
}

/// Provoca o pedido do sistema (microfone e, no Mac, Acessibilidade) e devolve a situação nova.
pub fn pedir(permissao: Permissao) -> Situacao {
    match permissao {
        Permissao::Microfone => sistema::pedir_microfone(),
        Permissao::Acessibilidade => sistema::pedir_acessibilidade(),
    }
    situacao(permissao)
}

/// Endereço da tela do sistema onde o usuário concede a permissão.
pub fn endereco_configuracoes(permissao: Permissao) -> Option<&'static str> {
    sistema::endereco(permissao)
}

/// Lê a saída de `reg query ... /v Value` e diz se o Windows bloqueou o acesso (`Deny`).
#[cfg_attr(target_os = "macos", allow(dead_code))]
fn registro_nega(saida: &str) -> bool {
    saida
        .lines()
        .any(|l| l.contains("REG_SZ") && l.split_whitespace().last() == Some("Deny"))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn registro_com_deny_e_negado() {
        let saida = "\r\nHKEY_CURRENT_USER\\...\\microphone\r\n    Value    REG_SZ    Deny\r\n";
        assert!(registro_nega(saida));
    }

    #[test]
    fn registro_com_allow_ou_vazio_nao_e_negado() {
        assert!(!registro_nega("    Value    REG_SZ    Allow\r\n"));
        assert!(!registro_nega(""));
        assert!(!registro_nega("ERRO: Não foi possível localizar a chave"));
    }

    #[test]
    fn tela_do_sistema_existe_para_o_microfone() {
        assert!(endereco_configuracoes(Permissao::Microfone).is_some());
    }

    #[test]
    fn acessibilidade_so_existe_no_mac() {
        let mac = cfg!(target_os = "macos");
        assert_eq!(
            endereco_configuracoes(Permissao::Acessibilidade).is_some(),
            mac
        );
        assert_eq!(
            situacao(Permissao::Acessibilidade) == Situacao::NaoSeAplica,
            !mac
        );
    }
}

use super::{Permissao, Situacao, registro_nega};
use std::os::windows::process::CommandExt;
use std::process::Command;

const BASE: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\CapabilityAccessManager\ConsentStore\microphone";
const SEM_JANELA: u32 = 0x0800_0000; // CREATE_NO_WINDOW

fn bloqueado_no_registro(chave: &str) -> bool {
    Command::new("reg")
        .args(["query", chave, "/v", "Value"])
        .creation_flags(SEM_JANELA)
        .output()
        .map(|o| registro_nega(&String::from_utf8_lossy(&o.stdout)))
        .unwrap_or(false)
}

/// O Windows não tem diálogo para programas desktop: a chave de privacidade ("Deny") ou a falta
/// de entrada disponível contam como negada.
pub fn microfone() -> Situacao {
    use cpal::traits::HostTrait;
    if bloqueado_no_registro(BASE) || bloqueado_no_registro(&format!(r"{BASE}\NonPackaged")) {
        return Situacao::Negada;
    }
    if cpal::default_host().default_input_device().is_some() {
        Situacao::Concedida
    } else {
        Situacao::Negada
    }
}

pub fn acessibilidade() -> Situacao {
    Situacao::NaoSeAplica
}

pub fn pedir_microfone() {}

pub fn detalhe() -> String {
    String::new()
}

pub fn endereco(permissao: Permissao) -> Option<&'static str> {
    match permissao {
        Permissao::Microfone => Some("ms-settings:privacy-microphone"),
        Permissao::Acessibilidade => None,
    }
}

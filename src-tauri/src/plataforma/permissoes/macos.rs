use super::{Permissao, Situacao};
use std::ffi::c_void;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn acessibilidade() -> Situacao {
    // SAFETY: função do sistema sem argumentos, só consulta.
    if unsafe { AXIsProcessTrusted() } {
        Situacao::Concedida
    } else {
        // O sistema não diferencia "nunca perguntei" de "negada" para Acessibilidade.
        Situacao::Negada
    }
}

pub fn microfone() -> Situacao {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject};
    #[link(name = "AVFoundation", kind = "framework")]
    unsafe extern "C" {
        static AVMediaTypeAudio: *const c_void;
    }
    let Some(classe) = AnyClass::get(c"AVCaptureDevice") else {
        return Situacao::Pendente;
    };
    // SAFETY: `authorizationStatusForMediaType:` é método de classe que devolve NSInteger.
    let estado: isize = unsafe {
        msg_send![classe, authorizationStatusForMediaType: AVMediaTypeAudio as *const AnyObject]
    };
    match estado {
        3 => Situacao::Concedida,
        1 | 2 => Situacao::Negada,
        _ => Situacao::Pendente,
    }
}

/// Abrir um fluxo de entrada faz o macOS mostrar o diálogo de microfone na primeira vez.
pub fn pedir_microfone() {
    crate::audio::captura::sondar_entrada();
}

pub fn endereco(permissao: Permissao) -> Option<&'static str> {
    Some(match permissao {
        Permissao::Microfone => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone"
        }
        Permissao::Acessibilidade => {
            "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility"
        }
    })
}

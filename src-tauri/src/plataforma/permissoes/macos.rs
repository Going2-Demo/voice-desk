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

/// Traduz o `AVAuthorizationStatus`; `confirmar` é a segunda opinião do sistema, só consultada
/// quando o estado lido é "negado" (nesse caso o pedido nunca abre diálogo).
fn traduzir_microfone(estado: isize, confirmar: impl FnOnce() -> bool) -> Situacao {
    match estado {
        3 => Situacao::Concedida,
        // O estado lido pelo processo pode ficar velho depois que o usuário liga o microfone em
        // Ajustes do Sistema; pedir de novo devolve a decisão atual sem mostrar diálogo.
        1 | 2 => {
            if confirmar() {
                Situacao::Concedida
            } else {
                Situacao::Negada
            }
        }
        _ => Situacao::Pendente,
    }
}

/// Pergunta ao sistema a decisão atual do usuário (só chamar com o estado já decidido).
fn decisao_atual_do_microfone(classe: &objc2::runtime::AnyClass, tipo: *const c_void) -> bool {
    use block2::RcBlock;
    use objc2::msg_send;
    use objc2::runtime::{AnyObject, Bool};
    use std::sync::mpsc;
    use std::time::Duration;

    let (envia, recebe) = mpsc::channel();
    let bloco = RcBlock::new(move |concedido: Bool| {
        let _ = envia.send(concedido.as_bool());
    });
    // SAFETY: `requestAccessForMediaType:completionHandler:` recebe um NSString e um bloco
    // `void (^)(BOOL)`; o bloco é copiado pelo sistema e vive até ser chamado.
    unsafe {
        let _: () = msg_send![
            classe,
            requestAccessForMediaType: tipo as *const AnyObject,
            completionHandler: &*bloco
        ];
    }
    recebe
        .recv_timeout(Duration::from_millis(500))
        .unwrap_or(false)
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
    // SAFETY: lê o símbolo constante do framework já ligado acima.
    traduzir_microfone(estado, || {
        decisao_atual_do_microfone(classe, unsafe { AVMediaTypeAudio })
    })
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

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn concedida_nao_consulta_o_sistema_de_novo() {
        assert_eq!(
            traduzir_microfone(3, || panic!("não devia consultar")),
            Situacao::Concedida
        );
    }

    #[test]
    fn negada_vira_concedida_quando_o_sistema_confirma_a_decisao_nova() {
        assert_eq!(traduzir_microfone(2, || true), Situacao::Concedida);
        assert_eq!(traduzir_microfone(1, || true), Situacao::Concedida);
    }

    #[test]
    fn negada_continua_negada_sem_confirmacao() {
        assert_eq!(traduzir_microfone(2, || false), Situacao::Negada);
    }

    #[test]
    fn pendente_nunca_pede_para_nao_abrir_o_dialogo_no_acompanhamento() {
        assert_eq!(
            traduzir_microfone(0, || panic!("não devia consultar")),
            Situacao::Pendente
        );
    }
}

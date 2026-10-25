use super::{Permissao, Situacao};
use std::ffi::c_void;

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn AXIsProcessTrusted() -> bool;
    fn AXUIElementCreateSystemWide() -> *const c_void;
    fn AXUIElementCopyAttributeValue(
        elemento: *const c_void,
        atributo: *const c_void,
        valor: *mut *const c_void,
    ) -> i32;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alocador: *const c_void,
        texto: *const std::ffi::c_char,
        codificacao: u32,
    ) -> *const c_void;
    fn CFRelease(objeto: *const c_void);
}

/// `kAXErrorAPIDisabled`: o sistema recusa chamadas de acessibilidade a este processo.
const AX_API_DESABILITADA: i32 = -25211;
const CF_UTF8: u32 = 0x0800_0100;

/// Traduz a resposta de `AXIsProcessTrusted`; `confirmar` é a segunda opinião, só consultada
/// quando ela diz "não" (o valor lido pelo processo pode ficar velho depois de conceder).
fn traduzir_acessibilidade(confiavel: bool, confirmar: impl FnOnce() -> bool) -> Situacao {
    if confiavel || confirmar() {
        Situacao::Concedida
    } else {
        // O sistema não diferencia "nunca perguntei" de "negada" para Acessibilidade.
        Situacao::Negada
    }
}

/// Faz uma chamada real de acessibilidade: sem permissão o sistema responde `APIDisabled`,
/// e essa resposta é sempre atual, ao contrário do cache de `AXIsProcessTrusted`.
fn acessibilidade_responde() -> bool {
    // SAFETY: cria e libera os objetos CF que usa; o atributo é uma constante ASCII.
    unsafe {
        let sistema = AXUIElementCreateSystemWide();
        if sistema.is_null() {
            return false;
        }
        let atributo =
            CFStringCreateWithCString(std::ptr::null(), c"AXFocusedApplication".as_ptr(), CF_UTF8);
        if atributo.is_null() {
            CFRelease(sistema);
            return false;
        }
        let mut valor: *const c_void = std::ptr::null();
        let erro = AXUIElementCopyAttributeValue(sistema, atributo, &mut valor);
        if !valor.is_null() {
            CFRelease(valor);
        }
        CFRelease(atributo);
        CFRelease(sistema);
        erro != AX_API_DESABILITADA
            && (erro == 0 || erro == AX_SEM_VALOR || erro == AX_ATRIBUTO_NAO_SUPORTADO)
    }
}

const AX_SEM_VALOR: i32 = -25212;
const AX_ATRIBUTO_NAO_SUPORTADO: i32 = -25205;

pub fn acessibilidade() -> Situacao {
    // SAFETY: função do sistema sem argumentos, só consulta.
    let confiavel = unsafe { AXIsProcessTrusted() };
    traduzir_acessibilidade(confiavel, acessibilidade_responde)
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
    fn acessibilidade_confiavel_nao_consulta_o_sistema_de_novo() {
        assert_eq!(
            traduzir_acessibilidade(true, || panic!("não devia consultar")),
            Situacao::Concedida
        );
    }

    #[test]
    fn acessibilidade_velha_vira_concedida_quando_o_sistema_responde() {
        assert_eq!(traduzir_acessibilidade(false, || true), Situacao::Concedida);
    }

    #[test]
    fn acessibilidade_continua_negada_sem_resposta_do_sistema() {
        assert_eq!(traduzir_acessibilidade(false, || false), Situacao::Negada);
    }

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

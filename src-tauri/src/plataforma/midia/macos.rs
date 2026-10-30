//! Mac: MediaRemote (framework privado). Carregado com `dlopen`; se faltar ou o sistema recusar
//! (macOS recentes restringem apps de terceiros), a consulta "está tocando?" não responde e nada é
//! pausado.

use super::{Controle, Pausada};
use block2::RcBlock;
use objc2::runtime::Bool;
use std::ffi::{CStr, c_char, c_int, c_void};
use std::sync::mpsc;
use std::time::Duration;

const CAMINHO: &CStr = c"/System/Library/PrivateFrameworks/MediaRemote.framework/MediaRemote";
const COMANDO_TOCAR: c_int = 0;
const COMANDO_PAUSAR: c_int = 1;
const ESPERA: Duration = Duration::from_millis(250);

unsafe extern "C" {
    fn dlopen(caminho: *const c_char, modo: c_int) -> *mut c_void;
    fn dlsym(lib: *mut c_void, nome: *const c_char) -> *mut c_void;
    fn dispatch_get_global_queue(identificador: isize, flags: usize) -> *mut c_void;
}

type Consulta = unsafe extern "C" fn(*mut c_void, &block2::Block<dyn Fn(Bool)>);
type Enviar = unsafe extern "C" fn(c_int, *const c_void) -> bool;

fn simbolo(nome: &CStr) -> Option<*mut c_void> {
    // RTLD_LAZY = 1
    let lib = unsafe { dlopen(CAMINHO.as_ptr(), 1) };
    if lib.is_null() {
        return None;
    }
    let p = unsafe { dlsym(lib, nome.as_ptr()) };
    (!p.is_null()).then_some(p)
}

fn tocando() -> bool {
    let Some(p) = simbolo(c"MRMediaRemoteGetNowPlayingApplicationIsPlaying") else {
        return false;
    };
    let consulta: Consulta = unsafe { std::mem::transmute(p) };
    let (tx, rx) = mpsc::channel();
    let bloco: RcBlock<dyn Fn(Bool)> = RcBlock::new(move |t: Bool| {
        let _ = tx.send(t.as_bool());
    });
    unsafe { consulta(dispatch_get_global_queue(0, 0), &bloco) };
    rx.recv_timeout(ESPERA).unwrap_or(false)
}

fn enviar(comando: c_int) -> bool {
    let Some(p) = simbolo(c"MRMediaRemoteSendCommand") else {
        return false;
    };
    let enviar: Enviar = unsafe { std::mem::transmute(p) };
    unsafe { enviar(comando, std::ptr::null()) }
}

pub struct MediaRemote;

impl Controle for MediaRemote {
    fn pausar_tocando(&self) -> Vec<Pausada> {
        if tocando() && enviar(COMANDO_PAUSAR) {
            vec![Pausada(String::new())]
        } else {
            Vec::new()
        }
    }

    fn retomar(&self, _pausadas: &[Pausada]) {
        enviar(COMANDO_TOCAR);
    }
}

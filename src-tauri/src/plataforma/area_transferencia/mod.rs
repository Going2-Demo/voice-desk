//! Área de transferência do sistema (texto e imagem). Formatos além desses (ex.: arquivos
//! copiados no Finder ou Explorer) não são preservados na v1 (ADR-07).

use arboard::{Clipboard, ImageData};
use std::borrow::Cow;

/// Conteúdo que estava na área de transferência, guardado para ser devolvido depois.
#[derive(Debug, Clone, PartialEq)]
pub enum Conteudo {
    Vazio,
    Texto(String),
    Imagem {
        largura: usize,
        altura: usize,
        rgba: Vec<u8>,
    },
}

fn abrir() -> Result<Clipboard, String> {
    Clipboard::new().map_err(|e| e.to_string())
}

/// Lê o conteúdo atual; qualquer falha ou formato desconhecido vira `Vazio`.
pub fn ler() -> Conteudo {
    let Ok(mut area) = abrir() else {
        return Conteudo::Vazio;
    };
    if let Ok(texto) = area.get_text() {
        return Conteudo::Texto(texto);
    }
    match area.get_image() {
        Ok(img) => Conteudo::Imagem {
            largura: img.width,
            altura: img.height,
            rgba: img.bytes.into_owned(),
        },
        Err(_) => Conteudo::Vazio,
    }
}

pub fn escrever_texto(texto: &str) -> Result<(), String> {
    abrir()?.set_text(texto).map_err(|e| e.to_string())
}

/// Devolve o conteúdo guardado antes do ditado (`Vazio` limpa a área).
pub fn restaurar(conteudo: &Conteudo) -> Result<(), String> {
    let mut area = abrir()?;
    match conteudo {
        Conteudo::Vazio => area.clear().map_err(|e| e.to_string()),
        Conteudo::Texto(t) => area.set_text(t.as_str()).map_err(|e| e.to_string()),
        Conteudo::Imagem {
            largura,
            altura,
            rgba,
        } => area
            .set_image(ImageData {
                width: *largura,
                height: *altura,
                bytes: Cow::Borrowed(rgba),
            })
            .map_err(|e| e.to_string()),
    }
}

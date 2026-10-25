//! O macOS só mostra o diálogo de microfone (e só lista o app em Privacidade e
//! Segurança > Microfone) se o Info.plist do bundle tiver a descrição de uso.

use std::fs;

fn ler(arquivo: &str) -> String {
    let caminho = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), arquivo);
    fs::read_to_string(&caminho).unwrap_or_else(|e| panic!("não leu {caminho}: {e}"))
}

fn valor_da_chave<'a>(plist: &'a str, chave: &str) -> Option<&'a str> {
    let depois = plist.split(&format!("<key>{chave}</key>")).nth(1)?;
    Some(depois.trim_start())
}

#[test]
fn info_plist_descreve_o_uso_do_microfone() {
    let plist = ler("Info.plist");
    let resto = valor_da_chave(&plist, "NSMicrophoneUsageDescription")
        .expect("Info.plist sem NSMicrophoneUsageDescription");
    let texto = resto
        .strip_prefix("<string>")
        .and_then(|r| r.split("</string>").next())
        .expect("NSMicrophoneUsageDescription precisa ser <string>");
    assert!(!texto.trim().is_empty(), "descrição de uso vazia");
}

#[test]
fn entitlements_permitem_entrada_de_audio() {
    let plist = ler("Entitlements.plist");
    let resto = valor_da_chave(&plist, "com.apple.security.device.audio-input")
        .expect("Entitlements sem com.apple.security.device.audio-input");
    assert!(resto.starts_with("<true/>"));
}

#[test]
fn tauri_conf_aponta_os_entitlements() {
    let conf = ler("tauri.conf.json");
    assert!(conf.contains("\"entitlements\": \"Entitlements.plist\""));
}

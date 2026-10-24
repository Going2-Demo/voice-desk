//! Embute os arquivos de `migracoes/` (ordem pelo nome) em `$OUT_DIR/migracoes.rs` e prepara o
//! manifesto do Windows.

use std::{env, fs, path::PathBuf};

fn main() {
    let pasta = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("migracoes");
    println!("cargo:rerun-if-changed=migracoes");
    // O commit embutido (`option_env!` em comandos.rs) precisa invalidar o cache do cargo: sem
    // isto, num build com cache restaurado um dos alvos do instalador universal do Mac saiu com
    // o commit antigo (ENG-68). Repassar por `rustc-env` muda a linha de comando do rustc.
    println!("cargo:rerun-if-env-changed=VOICE_DESK_COMMIT");
    let commit = env::var("VOICE_DESK_COMMIT").unwrap_or_else(|_| "local".into());
    println!("cargo:rustc-env=VOICE_DESK_COMMIT={commit}");
    let mut nomes: Vec<String> = fs::read_dir(&pasta)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".sql"))
        .collect();
    nomes.sort();
    let itens: Vec<String> = nomes
        .iter()
        .map(|n| {
            format!(
                "    ({n:?}, include_str!({:?})),",
                pasta.join(n).display().to_string()
            )
        })
        .collect();
    let codigo = format!(
        "pub const MIGRACOES: &[(&str, &str)] = &[\n{}\n];\n",
        itens.join("\n")
    );
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("migracoes.rs"),
        codigo,
    )
    .unwrap();

    if windows_msvc() {
        // O manifesto com Common Controls v6 vai por argumento do linker para TODOS os alvos
        // (app, lib e executável de teste da lib). Sem ele, o teste da lib nem inicia
        // (STATUS_ENTRYPOINT_NOT_FOUND). O tauri_build não embute o dele para não duplicar.
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
        let janelas = tauri_build::WindowsAttributes::new_without_app_manifest();
        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(janelas)).unwrap();
    } else {
        tauri_build::build()
    }
}

fn windows_msvc() -> bool {
    env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
}

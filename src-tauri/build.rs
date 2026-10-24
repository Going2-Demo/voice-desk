//! Embute os arquivos de `migracoes/` (ordem pelo nome) em `$OUT_DIR/migracoes.rs`.

use std::{env, fs, path::PathBuf};

fn main() {
    let pasta = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join("migracoes");
    println!("cargo:rerun-if-changed=migracoes");
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
    embutir_manifesto_nos_testes();
    tauri_build::build()
}

/// No Windows, o executável de teste da lib não leva o manifesto que o `tauri_build` embute só no
/// app. Sem Common Controls v6 (importado pelo Tauri/muda), ele nem inicia (STATUS_ENTRYPOINT_NOT_FOUND).
fn embutir_manifesto_nos_testes() {
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        println!("cargo:rustc-link-arg-tests=/MANIFEST:EMBED");
        println!(
            "cargo:rustc-link-arg-tests=/MANIFESTDEPENDENCY:type='win32' name='Microsoft.Windows.Common-Controls' version='6.0.0.0' processorArchitecture='*' publicKeyToken='6595b64144ccf1df' language='*'"
        );
    }
}

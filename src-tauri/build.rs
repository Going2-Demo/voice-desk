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
    tauri_build::build()
}

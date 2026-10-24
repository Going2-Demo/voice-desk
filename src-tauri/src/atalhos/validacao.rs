//! Validação do atalho global (RF-01.1): precisa de modificador, não pode ser só uma letra nem
//! um atalho reservado do sistema. Formato canônico: `Ctrl+Alt+Space` (partes separadas por `+`).

use crate::nucleo::ErroApp;

const MODIFICADORES: &[&str] = &["Ctrl", "Alt", "Shift", "Cmd"];

/// Combinações reservadas pelo sistema (já em forma canônica).
const RESERVADOS_MAC: &[&str] = &[
    "Cmd+Q",
    "Cmd+W",
    "Cmd+H",
    "Cmd+M",
    "Cmd+Tab",
    "Cmd+Space",
    "Cmd+C",
    "Cmd+V",
    "Cmd+X",
    "Cmd+Z",
    "Cmd+A",
    "Cmd+S",
    "Cmd+Alt+Esc",
    "Ctrl+Cmd+Q",
];
const RESERVADOS_WINDOWS: &[&str] = &[
    "Alt+F4",
    "Alt+Tab",
    "Ctrl+Alt+Delete",
    "Ctrl+Shift+Esc",
    "Ctrl+Esc",
    "Cmd+L",
    "Cmd+D",
    "Cmd+E",
    "Cmd+R",
    "Cmd+Tab",
    "Ctrl+C",
    "Ctrl+V",
    "Ctrl+X",
    "Ctrl+Z",
    "Ctrl+A",
    "Ctrl+S",
    "Ctrl+W",
];

fn recusa(mensagem: &str) -> ErroApp {
    ErroApp::novo("atalho_invalido", mensagem).com_campo("combinacao")
}

fn modificador_canonico(parte: &str) -> Option<&'static str> {
    match parte.to_lowercase().as_str() {
        "ctrl" | "control" | "ctl" => Some("Ctrl"),
        "alt" | "option" | "opt" => Some("Alt"),
        "shift" => Some("Shift"),
        "cmd" | "command" | "super" | "meta" | "win" | "windows" => Some("Cmd"),
        _ => None,
    }
}

fn tecla_canonica(parte: &str) -> Option<String> {
    let minuscula = parte.to_lowercase();
    let nome = match minuscula.as_str() {
        "space" | "espaco" | "espaço" => "Space".to_string(),
        "esc" | "escape" => "Esc".to_string(),
        "enter" | "return" => "Enter".to_string(),
        "tab" => "Tab".to_string(),
        "backspace" => "Backspace".to_string(),
        "delete" | "del" => "Delete".to_string(),
        "insert" => "Insert".to_string(),
        "home" => "Home".to_string(),
        "end" => "End".to_string(),
        "pageup" => "PageUp".to_string(),
        "pagedown" => "PageDown".to_string(),
        "up" | "arrowup" => "Up".to_string(),
        "down" | "arrowdown" => "Down".to_string(),
        "left" | "arrowleft" => "Left".to_string(),
        "right" | "arrowright" => "Right".to_string(),
        t if t.len() == 1 && t.chars().all(|c| c.is_ascii_alphanumeric()) => t.to_uppercase(),
        t if t.len() <= 3
            && t.starts_with('f')
            && t[1..].parse::<u8>().is_ok_and(|n| (1..=24).contains(&n)) =>
        {
            t.to_uppercase()
        }
        _ => return None,
    };
    Some(nome)
}

/// Devolve a forma canônica (`Ctrl+Alt+Space`) ou o motivo da recusa, já em português.
pub fn validar(bruto: &str, plataforma: &str) -> Result<String, ErroApp> {
    let partes: Vec<&str> = bruto.split('+').map(str::trim).collect();
    if bruto.trim().is_empty() || partes.iter().any(|p| p.is_empty()) {
        return Err(recusa(
            "Digite uma combinação, por exemplo Ctrl+Alt+Espaço.",
        ));
    }
    let mut modificadores: Vec<&'static str> = Vec::new();
    let mut tecla: Option<String> = None;
    for parte in &partes {
        if let Some(m) = modificador_canonico(parte) {
            if !modificadores.contains(&m) {
                modificadores.push(m);
            }
        } else if let Some(t) = tecla_canonica(parte) {
            if tecla.replace(t).is_some() {
                return Err(recusa("Use uma única tecla além dos modificadores."));
            }
        } else {
            return Err(recusa(&format!(
                "A tecla “{parte}” não é aceita num atalho."
            )));
        }
    }
    let Some(tecla) = tecla else {
        return Err(recusa(
            "Falta a tecla principal (por exemplo, Espaço ou D).",
        ));
    };
    if modificadores.is_empty() {
        return Err(recusa(
            "O atalho precisa de pelo menos uma tecla modificadora (Ctrl, Alt, Shift ou ⌘).",
        ));
    }
    if tecla == "Esc" {
        return Err(recusa(
            "Esc é usado para cancelar o ditado; escolha outra tecla.",
        ));
    }
    let so_shift = modificadores == ["Shift"];
    let tecla_de_texto = tecla.len() == 1 || matches!(tecla.as_str(), "Space" | "Enter");
    if so_shift && tecla_de_texto {
        return Err(recusa(
            "Shift com uma letra ou número é digitação normal; combine com Ctrl, Alt ou ⌘.",
        ));
    }
    modificadores.sort_by_key(|m| MODIFICADORES.iter().position(|x| x == m));
    let canonica = format!("{}+{}", modificadores.join("+"), tecla);
    let reservados = if plataforma == "macos" {
        RESERVADOS_MAC
    } else {
        RESERVADOS_WINDOWS
    };
    if reservados.contains(&canonica.as_str()) {
        return Err(recusa(&format!(
            "{} já é usado pelo sistema; escolha outra combinação.",
            exibir(&canonica, plataforma)
        )));
    }
    Ok(canonica)
}

/// Texto para mostrar ao usuário (`⌃⌥Espaço` no Mac, `Ctrl+Alt+Espaço` no Windows).
pub fn exibir(canonica: &str, plataforma: &str) -> String {
    let partes = canonica.split('+').map(|p| {
        let tecla = if p == "Space" { "Espaço" } else { p };
        if plataforma == "macos" {
            match p {
                "Ctrl" => "⌃",
                "Alt" => "⌥",
                "Shift" => "⇧",
                "Cmd" => "⌘",
                _ => tecla,
            }
        } else if p == "Cmd" {
            "Win"
        } else {
            tecla
        }
    });
    if plataforma == "macos" {
        partes.collect()
    } else {
        partes.collect::<Vec<_>>().join("+")
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn aceita_e_normaliza_a_combinacao() {
        assert_eq!(
            validar("alt+ctrl+espaço", "windows").unwrap(),
            "Ctrl+Alt+Space"
        );
        assert_eq!(validar("Cmd + Shift + d", "macos").unwrap(), "Shift+Cmd+D");
    }

    #[test]
    fn recusa_sem_modificador() {
        let e = validar("Space", "windows").unwrap_err();
        assert_eq!(e.codigo, "atalho_invalido");
        assert!(e.mensagem.contains("modificadora"));
        assert_eq!(e.campo.as_deref(), Some("combinacao"));
    }

    #[test]
    fn recusa_so_uma_letra() {
        assert!(validar("A", "macos").is_err());
        assert!(
            validar("Shift+A", "macos")
                .unwrap_err()
                .mensagem
                .contains("digitação")
        );
    }

    #[test]
    fn recusa_atalho_do_sistema() {
        let e = validar("Alt+F4", "windows").unwrap_err();
        assert!(e.mensagem.contains("usado pelo sistema"));
        assert!(validar("Cmd+Space", "macos").is_err());
        assert!(validar("Cmd+Space", "windows").is_ok());
    }

    #[test]
    fn recusa_esc_e_entradas_malformadas() {
        assert!(validar("Ctrl+Esc", "macos").is_err());
        assert!(validar("", "macos").is_err());
        assert!(validar("Ctrl+", "macos").is_err());
        assert!(
            validar("Ctrl+Alt", "macos")
                .unwrap_err()
                .mensagem
                .contains("principal")
        );
        assert!(validar("Ctrl+A+B", "macos").is_err());
        assert!(validar("Ctrl+Ç", "macos").is_err());
    }

    #[test]
    fn teclas_de_funcao_valem_com_modificador() {
        assert_eq!(validar("ctrl+f9", "windows").unwrap(), "Ctrl+F9");
        assert!(validar("ctrl+f99", "windows").is_err());
    }

    #[test]
    fn exibe_por_plataforma() {
        assert_eq!(exibir("Ctrl+Alt+Space", "macos"), "⌃⌥Espaço");
        assert_eq!(exibir("Ctrl+Alt+Space", "windows"), "Ctrl+Alt+Espaço");
    }
}

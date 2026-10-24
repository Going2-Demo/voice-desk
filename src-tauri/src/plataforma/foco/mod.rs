//! Programa em foco no momento em que o ditado começa (F-01). Só o nome do aplicativo é lido,
//! nunca o título da janela (pode conter texto do usuário).

/// Nome do aplicativo em foco, se o sistema informar.
pub fn programa_em_foco() -> Option<String> {
    let janela = active_win_pos_rs::get_active_window().ok()?;
    nome_valido(&janela.app_name)
}

fn nome_valido(nome: &str) -> Option<String> {
    let limpo = nome.trim();
    (!limpo.is_empty()).then(|| limpo.to_string())
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn nome_vazio_ou_so_espacos_vira_none() {
        assert_eq!(nome_valido("  "), None);
        assert_eq!(nome_valido(""), None);
        assert_eq!(nome_valido(" Notepad "), Some("Notepad".into()));
    }
}

//! Limpeza sem IA (F-09): tira palavras de preenchimento e formata. Funções puras, sem banco.
//!
//! Uma entrada da lista que termina em reticências ("é…", "um…") só some quando o texto traz as
//! reticências logo depois dela: é a hesitação, não o verbo "é" nem o artigo "um". As demais somem
//! onde aparecem como palavra inteira, sem diferenciar maiúsculas.

const RETICENCIAS: [&str; 2] = ["…", "..."];

struct Entrada {
    palavras: Vec<String>,
    exige_reticencias: bool,
}

/// "Minúsculas, espaços únicos, `...` vira `…`": a forma em que a lista é guardada e comparada.
pub fn normalizar_entrada(palavra: &str) -> String {
    palavra
        .replace("...", "…")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

fn entrada(palavra: &str) -> Option<Entrada> {
    let norm = normalizar_entrada(palavra);
    let exige_reticencias = norm.ends_with('…');
    let palavras: Vec<String> = norm
        .trim_end_matches('…')
        .split_whitespace()
        .map(str::to_string)
        .collect();
    if palavras.is_empty() {
        return None;
    }
    Some(Entrada {
        palavras,
        exige_reticencias,
    })
}

/// Pedaço do texto entre espaços: sinais antes, a palavra, sinais depois.
struct Ficha<'a> {
    inteira: &'a str,
    nucleo: String,
    sufixo: &'a str,
}

fn fichas(texto: &str) -> Vec<Ficha<'_>> {
    texto
        .split_whitespace()
        .map(|inteira| {
            let fim_nucleo = inteira
                .char_indices()
                .rev()
                .find(|(_, c)| c.is_alphanumeric())
                .map_or(0, |(i, c)| i + c.len_utf8());
            let (miolo, sufixo) = inteira.split_at(fim_nucleo);
            let nucleo = miolo
                .trim_start_matches(|c: char| !c.is_alphanumeric())
                .to_lowercase();
            // Sem letras (ex.: "—"), o sufixo é o próprio pedaço e ele nunca casa com a lista.
            Ficha {
                inteira,
                nucleo,
                sufixo: if fim_nucleo == 0 { inteira } else { sufixo },
            }
        })
        .collect()
}

fn casa(entrada: &Entrada, fichas: &[Ficha<'_>]) -> bool {
    let n = entrada.palavras.len();
    if fichas.len() < n {
        return false;
    }
    let janela = &fichas[..n];
    let palavras_iguais = janela
        .iter()
        .zip(&entrada.palavras)
        .all(|(f, p)| f.nucleo == *p);
    // Só a última palavra pode trazer sinal depois; no meio da expressão não há pausa.
    let sem_pausa_no_meio = janela[..n - 1].iter().all(|f| f.sufixo.is_empty());
    let reticencias_ok =
        !entrada.exige_reticencias || RETICENCIAS.iter().any(|r| janela[n - 1].sufixo.contains(r));
    palavras_iguais && sem_pausa_no_meio && reticencias_ok
}

/// Tira da transcrição as palavras da lista. Mantém `?` e `!` do trecho removido (passam para a
/// palavra anterior) e leva embora a vírgula e as reticências que acompanhavam a muleta.
pub fn remover_preenchimento(texto: &str, lista: &[String]) -> String {
    let mut entradas: Vec<Entrada> = lista.iter().filter_map(|p| entrada(p)).collect();
    // Expressões mais longas primeiro: "tipo assim" antes de "tipo".
    entradas.sort_by_key(|e| std::cmp::Reverse(e.palavras.len()));
    let todas = fichas(texto);
    let mut saida: Vec<String> = Vec::new();
    let mut i = 0;
    while i < todas.len() {
        let restantes = &todas[i..];
        let achada = entradas.iter().find(|e| casa(e, restantes));
        let Some(e) = achada else {
            saida.push(todas[i].inteira.to_string());
            i += 1;
            continue;
        };
        let ultima = &restantes[e.palavras.len() - 1];
        let marcas: String = ultima
            .sufixo
            .chars()
            .filter(|c| matches!(c, '?' | '!'))
            .collect();
        if !marcas.is_empty()
            && let Some(anterior) = saida.last_mut()
        {
            *anterior = format!("{}{marcas}", anterior.trim_end_matches([',', ';']));
        }
        i += e.palavras.len();
    }
    saida.join(" ")
}

/// RF-09.3: primeira letra maiúscula, ponto final se faltar pontuação, espaços duplos fora.
pub fn formatar(texto: &str) -> String {
    let compacto = texto.split_whitespace().collect::<Vec<_>>().join(" ");
    let sem_sobra = compacto
        .trim_end_matches([',', ';'])
        .trim_start_matches([',', ';']);
    let mut letras = sem_sobra.chars();
    let Some(primeira) = letras.next() else {
        return String::new();
    };
    let mut saida: String = primeira.to_uppercase().collect();
    saida.push_str(letras.as_str());
    let ultimo = saida.chars().next_back().unwrap_or(' ');
    if !matches!(ultimo, '.' | '!' | '?' | '…' | ':' | ')' | '"' | '”' | '»') {
        saida.push('.');
    }
    saida
}

/// Aplica o que o modo liga. Tudo desligado devolve a transcrição sem tocar em um caractere.
pub fn limpar(texto: &str, lista: &[String], remover: bool, formatar_texto: bool) -> String {
    let mut t = texto.to_string();
    if remover {
        t = remover_preenchimento(&t, lista);
    }
    if formatar_texto {
        t = formatar(&t);
    }
    t
}

#[cfg(test)]
mod testes {
    use super::*;

    fn lista(itens: &[&str]) -> Vec<String> {
        itens.iter().map(|s| s.to_string()).collect()
    }

    fn padrao() -> Vec<String> {
        lista(&["é…", "hã", "hum", "né", "tipo", "uh", "um…", "you know"])
    }

    #[test]
    fn tudo_desligado_devolve_a_transcricao_igual() {
        let t = "é…  eu acho que sim";
        assert_eq!(limpar(t, &padrao(), false, false), t);
    }

    #[test]
    fn chat_com_remocao_e_formatacao_do_criterio_de_aceite() {
        let r = limpar("hã… chego em dez minutos", &padrao(), true, true);
        assert_eq!(r, "Chego em dez minutos.");
    }

    #[test]
    fn so_remover_nao_formata_e_so_formatar_nao_remove() {
        assert_eq!(
            limpar("hã… chego em dez minutos", &padrao(), true, false),
            "chego em dez minutos"
        );
        assert_eq!(
            limpar("hã… chego em dez minutos", &padrao(), false, true),
            "Hã… chego em dez minutos."
        );
    }

    #[test]
    fn tipo_na_lista_tira_so_o_tipo_e_o_e_do_verbo_fica() {
        assert_eq!(limpar("é tipo assim", &padrao(), true, true), "É assim.");
    }

    #[test]
    fn e_com_reticencias_some_e_com_tres_pontos_tambem() {
        assert_eq!(
            limpar("é... eu acho que sim", &padrao(), true, true),
            "Eu acho que sim."
        );
        assert_eq!(
            limpar("é… eu acho que sim", &padrao(), true, true),
            "Eu acho que sim."
        );
        assert_eq!(limpar("isso é bom", &padrao(), true, false), "isso é bom");
    }

    #[test]
    fn formatacao_remove_espacos_duplos() {
        assert_eq!(limpar("olá  tudo bem", &[], false, true), "Olá tudo bem.");
    }

    #[test]
    fn nao_poe_ponto_se_ja_termina_em_pontuacao() {
        assert_eq!(formatar("vamos?"), "Vamos?");
        assert_eq!(formatar("certo!"), "Certo!");
        assert_eq!(formatar("espera…"), "Espera…");
        assert_eq!(formatar("já foi."), "Já foi.");
    }

    #[test]
    fn virgula_da_muleta_vai_junto_e_a_frase_fecha() {
        let r = limpar(
            "então, tipo, preciso falar com o Pereira né",
            &padrao(),
            true,
            true,
        );
        assert_eq!(r, "Então, preciso falar com o Pereira.");
    }

    #[test]
    fn interrogacao_da_muleta_passa_para_a_palavra_anterior() {
        let r = limpar("você vai, né?", &padrao(), true, true);
        assert_eq!(r, "Você vai?");
    }

    #[test]
    fn expressao_longa_ganha_da_curta_e_ignora_maiusculas() {
        let l = lista(&["tipo", "tipo assim"]);
        assert_eq!(remover_preenchimento("Foi Tipo Assim bom", &l), "Foi bom");
    }

    #[test]
    fn palavra_inteira_apenas_e_em_ingles_tambem() {
        let l = lista(&["um…", "hum", "you know"]);
        assert_eq!(remover_preenchimento("humor humano", &l), "humor humano");
        assert_eq!(remover_preenchimento("um dia", &l), "um dia");
        assert_eq!(
            remover_preenchimento("um… it was, you know, fine", &l),
            "it was, fine"
        );
    }

    #[test]
    fn lista_vazia_so_arruma_a_pontuacao_e_texto_vazio_fica_vazio() {
        assert_eq!(limpar("preciso sair", &[], true, true), "Preciso sair.");
        assert_eq!(limpar("", &padrao(), true, true), "");
        assert_eq!(limpar("hã…", &padrao(), true, true), "");
    }

    #[test]
    fn entrada_vazia_ou_so_reticencias_e_ignorada() {
        let l = lista(&["", "…", "  "]);
        assert_eq!(remover_preenchimento("oi tudo", &l), "oi tudo");
    }

    #[test]
    fn normalizar_entrada_junta_espacos_minusculas_e_reticencias() {
        assert_eq!(normalizar_entrada("  É...  "), "é…");
        assert_eq!(normalizar_entrada("You   Know"), "you know");
    }
}

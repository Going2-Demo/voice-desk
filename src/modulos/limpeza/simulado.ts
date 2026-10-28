/** Dados do modo navegador desta área: lista e opções em memória, com as regras do núcleo. */
import type { ApiLimpeza, OpcoesTexto, PalavraPreenchimento } from "./api";
import { limpar, normalizarEntrada } from "./algoritmo";
import { ErroLimpeza } from "./erro";

const LIMITE = 200;
const SEMENTE = ["é…", "hã", "ahn", "hum", "né", "tipo", "uh", "uhm", "um…", "you know"];
const PADRAO: OpcoesTexto = {
  idioma: "pt-BR",
  remover_preenchimento: true,
  formatar: true,
  enviar_enter: false,
};

let itens: PalavraPreenchimento[] = [];
let opcoes: OpcoesTexto = { ...PADRAO };
let proximo = 1;
const novoId = () => `f${proximo++}`;
const ordenar = (l: PalavraPreenchimento[]) =>
  [...l].sort((a, b) => normalizarEntrada(a.palavra).localeCompare(normalizarEntrada(b.palavra)));

export function reiniciarSimulado(
  palavras: string[] = SEMENTE,
  inicial: Partial<OpcoesTexto> = {},
) {
  proximo = 1;
  itens = palavras.map((palavra) => ({ id: novoId(), palavra }));
  opcoes = { ...PADRAO, ...inicial };
}
reiniciarSimulado();

export const simulado: ApiLimpeza = {
  async listar() {
    return { palavras: ordenar(itens), limite: LIMITE };
  },
  async adicionar(bruta) {
    const norm = normalizarEntrada(bruta);
    if (!norm.replace(/…/g, "").trim())
      throw new ErroLimpeza("limpeza_vazia", "Digite uma palavra ou expressão.", "palavra");
    if ([...norm].length > 40)
      throw new ErroLimpeza(
        "limpeza_palavra_longa",
        "Use até 40 caracteres por palavra.",
        "palavra",
      );
    const palavra = bruta.replaceAll("...", "…").split(/\s+/).filter(Boolean).join(" ");
    if (itens.some((i) => normalizarEntrada(i.palavra) === norm))
      throw new ErroLimpeza("limpeza_duplicada", `“${palavra}” já está na lista.`, "palavra");
    if (itens.length >= LIMITE)
      throw new ErroLimpeza("limpeza_limite", "Limite de 200 palavras.", "palavra");
    const nova = { id: novoId(), palavra };
    itens = [...itens, nova];
    return nova;
  },
  async remover(id) {
    itens = itens.filter((i) => i.id !== id);
  },
  async previa(texto) {
    if ([...texto].length > 2000)
      throw new ErroLimpeza("limpeza_texto_longo", "Use até 2.000 caracteres no teste.", "texto");
    return limpar(
      texto,
      itens.map((i) => i.palavra),
      opcoes.remover_preenchimento,
      opcoes.formatar,
    );
  },
  async opcoesObter() {
    return { ...opcoes };
  },
  async opcoesSalvar(novas) {
    opcoes = { ...novas };
    return { ...opcoes };
  },
};

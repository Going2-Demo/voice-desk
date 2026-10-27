/** Dados do modo navegador desta área: vocabulário em memória, com as mesmas regras do núcleo. */
import type { ApiVocabulario, Palavra } from "./api";
import { ErroVocabulario } from "./erro";

const LIMITE = 500;
const SEMENTE = ["Going2", "Hortolândia", "Kubernetes", "LGPD", "Dokploy", "Voice Desk", "SLA"];
const norm = (p: string) => p.trim().replace(/\s+/g, " ").toLowerCase();

let itens: Palavra[] = [];
let proximo = 1;
const novoId = () => `p${proximo++}`;

export function reiniciarSimulado(palavras: string[] = SEMENTE) {
  proximo = 1;
  itens = palavras.map((palavra) => ({ id: novoId(), palavra })).reverse();
}
reiniciarSimulado();

function validar(bruta: string, ignorarId?: string): string {
  const palavra = bruta.trim().replace(/\s+/g, " ");
  if (!palavra)
    throw new ErroVocabulario("vocabulario_vazio", "Digite uma palavra ou nome.", "palavra");
  if (palavra.length > 80)
    throw new ErroVocabulario(
      "vocabulario_palavra_longa",
      "Use até 80 caracteres por palavra.",
      "palavra",
    );
  if (itens.some((i) => i.id !== ignorarId && norm(i.palavra) === norm(palavra)))
    throw new ErroVocabulario(
      "vocabulario_duplicada",
      `“${palavra}” já está no vocabulário.`,
      "palavra",
    );
  return palavra;
}

export const simulado: ApiVocabulario = {
  async listar(busca) {
    const termo = norm(busca ?? "");
    return {
      palavras: itens.filter((i) => norm(i.palavra).includes(termo)),
      total: itens.length,
      limite: LIMITE,
      na_dica: itens.length,
    };
  },
  async adicionar(bruta) {
    const palavra = validar(bruta);
    if (itens.length >= LIMITE)
      throw new ErroVocabulario("vocabulario_limite", "Limite de 500 palavras.", "palavra");
    const nova = { id: novoId(), palavra };
    itens = [nova, ...itens];
    return nova;
  },
  async editar(id, bruta) {
    const palavra = validar(bruta, id);
    itens = itens.map((i) => (i.id === id ? { id, palavra } : i));
    return { id, palavra };
  },
  async remover(id) {
    itens = itens.filter((i) => i.id !== id);
  },
};

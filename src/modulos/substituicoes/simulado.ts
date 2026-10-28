/** Dados do modo navegador desta área: substituições em memória, com as mesmas regras do núcleo. */
import type { ApiSubstituicoes, Substituicao } from "./api";
import { ErroSubstituicoes } from "./erro";

const LIMITE = 200;
const SEMENTE: [string, string, boolean][] = [
  ["gou tu", "Going2", true],
  ["ele gê pê dê", "LGPD", true],
  ["ess el ei", "SLA", true],
  ["pra mim fazer", "para eu fazer", false],
  ["obrigadão", "muito obrigado", true],
];
const norm = (p: string) => p.trim().replace(/\s+/g, " ").toLowerCase();

let itens: Substituicao[] = [];
let proximo = 1;
const novoId = () => `s${proximo++}`;

export function reiniciarSimulado(pares: [string, string, boolean][] = SEMENTE) {
  proximo = 1;
  itens = pares.map(([original, substituto, ativa]) => ({
    id: novoId(),
    original,
    substituto,
    ativa,
  }));
  itens.reverse();
}
reiniciarSimulado();

function validar(o: string, s: string, ignorarId?: string): [string, string] {
  const original = o.trim().replace(/\s+/g, " ");
  const substituto = s.trim();
  if (!original)
    throw new ErroSubstituicoes(
      "substituicao_original_vazio",
      "Digite o que a transcrição costuma dizer.",
      "original",
    );
  if (!substituto)
    throw new ErroSubstituicoes(
      "substituicao_substituto_vazio",
      "Digite o texto que deve ser escrito no lugar.",
      "substituto",
    );
  if (original.length > LIMITE)
    throw new ErroSubstituicoes(
      "substituicao_texto_longo",
      "Use até 200 caracteres no original.",
      "original",
    );
  if (substituto.length > LIMITE)
    throw new ErroSubstituicoes(
      "substituicao_texto_longo",
      "Use até 200 caracteres no substituto.",
      "substituto",
    );
  if (itens.some((i) => i.id !== ignorarId && norm(i.original) === norm(original)))
    throw new ErroSubstituicoes(
      "substituicao_duplicada",
      `Já existe uma substituição para “${original}”.`,
      "original",
    );
  return [original, substituto];
}

const achar = (id: string) => {
  const i = itens.find((x) => x.id === id);
  if (!i)
    throw new ErroSubstituicoes(
      "substituicao_nao_encontrada",
      "Essa substituição não existe mais.",
    );
  return i;
};

export const simulado: ApiSubstituicoes = {
  async listar() {
    return [...itens];
  },
  async adicionar(o, s) {
    const [original, substituto] = validar(o, s);
    const nova = { id: novoId(), original, substituto, ativa: true };
    itens = [nova, ...itens];
    return nova;
  },
  async editar(id, o, s) {
    const atual = achar(id);
    const [original, substituto] = validar(o, s, id);
    const nova = { id, original, substituto, ativa: atual.ativa };
    itens = itens.map((i) => (i.id === id ? nova : i));
    return nova;
  },
  async ativar(id, ativa) {
    const nova = { ...achar(id), ativa };
    itens = itens.map((i) => (i.id === id ? nova : i));
    return nova;
  },
  async remover(id) {
    itens = itens.filter((i) => i.id !== id);
  },
};

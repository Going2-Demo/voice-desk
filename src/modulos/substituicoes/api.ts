import { commands, type Substituicao } from "../../ipc/bindings";
import { ErroSubstituicoes } from "./erro";
import { simulado } from "./simulado";

export type { Substituicao };

export type ApiSubstituicoes = {
  listar(): Promise<Substituicao[]>;
  adicionar(original: string, substituto: string): Promise<Substituicao>;
  editar(id: string, original: string, substituto: string): Promise<Substituicao>;
  ativar(id: string, ativa: boolean): Promise<Substituicao>;
  remover(id: string): Promise<void>;
};

type Resultado<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: { codigo: string; mensagem: string; campo?: string } };

async function desembrulhar<T>(r: Promise<Resultado<T>>): Promise<T> {
  const resposta = await r;
  if (resposta.status === "error") {
    const { codigo, mensagem, campo } = resposta.error;
    throw new ErroSubstituicoes(codigo, mensagem, campo);
  }
  return resposta.data;
}

const nucleo: ApiSubstituicoes = {
  listar: () => desembrulhar(commands.substituicoesListar()),
  adicionar: (o, s) => desembrulhar(commands.substituicoesAdicionar(o, s)),
  editar: (id, o, s) => desembrulhar(commands.substituicoesEditar(id, o, s)),
  ativar: (id, ativa) => desembrulhar(commands.substituicoesAtivar(id, ativa)),
  remover: async (id) => {
    await desembrulhar(commands.substituicoesRemover(id));
  },
};

/** No modo navegador (`VITE_IPC=simulado`) o núcleo é simulado; no app, vale o núcleo em Rust. */
export const api: ApiSubstituicoes = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

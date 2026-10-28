import {
  commands,
  type ListaPreenchimento,
  type OpcoesTexto,
  type PalavraPreenchimento,
} from "../../ipc/bindings";
import { ErroLimpeza } from "./erro";
import { simulado } from "./simulado";

export type { ListaPreenchimento, OpcoesTexto, PalavraPreenchimento };

export type ApiLimpeza = {
  listar(): Promise<ListaPreenchimento>;
  adicionar(palavra: string): Promise<PalavraPreenchimento>;
  remover(id: string): Promise<void>;
  previa(texto: string): Promise<string>;
  opcoesObter(): Promise<OpcoesTexto>;
  opcoesSalvar(opcoes: OpcoesTexto): Promise<OpcoesTexto>;
};

type Resultado<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: { codigo: string; mensagem: string; campo?: string } };

async function desembrulhar<T>(r: Promise<Resultado<T>>): Promise<T> {
  const resposta = await r;
  if (resposta.status === "error") {
    const { codigo, mensagem, campo } = resposta.error;
    throw new ErroLimpeza(codigo, mensagem, campo);
  }
  return resposta.data;
}

const nucleo: ApiLimpeza = {
  listar: () => desembrulhar(commands.limpezaListar()),
  adicionar: (palavra) => desembrulhar(commands.limpezaAdicionar(palavra)),
  remover: async (id) => {
    await desembrulhar(commands.limpezaRemover(id));
  },
  previa: (texto) => desembrulhar(commands.limpezaPrevia(texto)),
  opcoesObter: () => desembrulhar(commands.limpezaOpcoesObter()),
  opcoesSalvar: (opcoes) => desembrulhar(commands.limpezaOpcoesSalvar(opcoes)),
};

/** No modo navegador (`VITE_IPC=simulado`) o núcleo é simulado; no app, vale o núcleo em Rust. */
export const api: ApiLimpeza = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

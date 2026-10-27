import { commands, type ListaVocabulario, type Palavra } from "../../ipc/bindings";
import { ErroVocabulario } from "./erro";
import { simulado } from "./simulado";

export type { ListaVocabulario, Palavra };

export type ApiVocabulario = {
  listar(busca: string | null): Promise<ListaVocabulario>;
  adicionar(palavra: string): Promise<Palavra>;
  editar(id: string, palavra: string): Promise<Palavra>;
  remover(id: string): Promise<void>;
};

type Resultado<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: { codigo: string; mensagem: string; campo?: string } };

async function desembrulhar<T>(r: Promise<Resultado<T>>): Promise<T> {
  const resposta = await r;
  if (resposta.status === "error") {
    const { codigo, mensagem, campo } = resposta.error;
    throw new ErroVocabulario(codigo, mensagem, campo);
  }
  return resposta.data;
}

const nucleo: ApiVocabulario = {
  listar: (busca) => desembrulhar(commands.vocabularioListar(busca)),
  adicionar: (palavra) => desembrulhar(commands.vocabularioAdicionar(palavra)),
  editar: (id, palavra) => desembrulhar(commands.vocabularioEditar(id, palavra)),
  remover: async (id) => {
    await desembrulhar(commands.vocabularioRemover(id));
  },
};

/** No modo navegador (`VITE_IPC=simulado`) o núcleo é simulado; no app, vale o núcleo em Rust. */
export const api: ApiVocabulario = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

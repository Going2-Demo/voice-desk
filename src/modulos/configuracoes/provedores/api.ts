import { commands, type EstadoChave_Serialize, type Provedor } from "../../../ipc/bindings";
import { ErroChave } from "./erro";
import { simulado } from "./simulado";

export type EstadoChave = EstadoChave_Serialize;

export type ApiChaves = {
  estado(): Promise<EstadoChave[]>;
  salvar(provedor: Provedor, chave: string): Promise<EstadoChave>;
  testar(provedor: Provedor): Promise<EstadoChave>;
  apagar(provedor: Provedor): Promise<EstadoChave>;
  abrirPagina(provedor: Provedor): Promise<void>;
};

type Resultado<T> =
  | { status: "ok"; data: T }
  | { status: "error"; error: { codigo: string; mensagem: string; campo?: string } };

async function desembrulhar<T>(r: Promise<Resultado<T>>): Promise<T> {
  const resposta = await r;
  if (resposta.status === "error") {
    const { codigo, mensagem, campo } = resposta.error;
    throw new ErroChave(codigo, mensagem, campo);
  }
  return resposta.data;
}

const nucleo: ApiChaves = {
  estado: () => desembrulhar(commands.chavesEstado()),
  salvar: (p, chave) => desembrulhar(commands.chavesSalvar(p, chave)),
  testar: (p) => desembrulhar(commands.chavesTestar(p)),
  apagar: (p) => desembrulhar(commands.chavesApagar(p)),
  abrirPagina: async (p) => {
    await desembrulhar(commands.chavesAbrirPagina(p));
  },
};

/** No modo navegador (`VITE_IPC=simulado`) o núcleo é simulado; no app, vale o núcleo em Rust. */
export const api: ApiChaves = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

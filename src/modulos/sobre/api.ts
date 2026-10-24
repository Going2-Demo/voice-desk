import { commands, type Sobre } from "../../ipc/bindings";
import { simulado } from "./simulado";

export type { Sobre };

export type ApiSobre = {
  obter(): Promise<Sobre>;
  abrirProjeto(): Promise<void>;
};

const nucleo: ApiSobre = {
  async obter() {
    const r = await commands.appSobre();
    if (r.status === "error") throw new Error(r.error.mensagem);
    return r.data;
  },
  async abrirProjeto() {
    const r = await commands.appAbrirProjeto();
    if (r.status === "error") throw new Error(r.error.mensagem);
  },
};

/** No modo navegador (`VITE_IPC=simulado`) o núcleo é simulado; no app, vale o núcleo em Rust. */
export const api: ApiSobre = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

import {
  commands,
  type EstadoPermissoes,
  type PermissaoApp as Permissao,
} from "../../../ipc/bindings";
import { desembrulhar } from "../../../ipc/resultado";
import { simulado } from "./simulado";

export type { EstadoPermissoes, Permissao };

export type ApiPermissoes = {
  estado(): Promise<EstadoPermissoes>;
  /** Provoca o pedido do sistema, quando existe, e devolve o estado novo. */
  pedir(permissao: Permissao): Promise<EstadoPermissoes>;
  abrirConfiguracoes(permissao: Permissao): Promise<void>;
};

const nucleo: ApiPermissoes = {
  estado: () => desembrulhar(commands.permissoesEstado()),
  pedir: (p) => desembrulhar(commands.permissoesPedir(p)),
  abrirConfiguracoes: async (p) => {
    await desembrulhar(commands.permissoesAbrirConfiguracoes(p));
  },
};

export const api: ApiPermissoes = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

import { commands, type EstadoPrimeiroUso } from "../../ipc/bindings";
import { desembrulhar } from "../../ipc/resultado";
import { simulado } from "./simulado";

export type { EstadoPrimeiroUso };

export type Passo = "permissoes" | "atalho" | "provedor" | "sincronizacao";

export type ApiPrimeiroUso = {
  estado(): Promise<EstadoPrimeiroUso>;
  pular(passo: Passo): Promise<EstadoPrimeiroUso>;
  /** Tira a marca de "pulado" quando o usuário faz o passo. */
  cumprir(passo: Passo): Promise<EstadoPrimeiroUso>;
  concluir(): Promise<EstadoPrimeiroUso>;
  /** Só no modo navegador: não há atalho global nem colagem, então o teste é simulado. */
  simularDitado?: () => string;
};

const nucleo: ApiPrimeiroUso = {
  estado: () => desembrulhar(commands.primeiroUsoEstado()),
  pular: (p) => desembrulhar(commands.primeiroUsoPular(p)),
  cumprir: (p) => desembrulhar(commands.primeiroUsoCumprir(p)),
  concluir: () => desembrulhar(commands.primeiroUsoConcluir()),
};

export const api: ApiPrimeiroUso = import.meta.env.VITE_IPC === "simulado" ? simulado : nucleo;

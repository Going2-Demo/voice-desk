import type { ErroApp_Serialize } from "../../ipc/bindings";

type Resultado<T> = Promise<
  { status: "ok"; data: T } | { status: "error"; error: ErroApp_Serialize }
>;

let iniciarComOSistema = true;

/** Modo navegador (VITE_IPC=simulado): mesmo formato de `commands`, estado só em memória. */
export const simulado = {
  bandejaIniciarComOSistema: (): Resultado<boolean> =>
    Promise.resolve({ status: "ok", data: iniciarComOSistema }),
  bandejaDefinirIniciarComOSistema: (ligado: boolean): Resultado<boolean> => {
    iniciarComOSistema = ligado;
    return Promise.resolve({ status: "ok", data: ligado });
  },
};

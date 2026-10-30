import type { EstadoMicrofone, ErroApp_Serialize } from "../../../ipc/bindings";

type Resultado<T> = Promise<
  { status: "ok"; data: T } | { status: "error"; error: ErroApp_Serialize }
>;

const CONECTADOS = ["MacBook Pro — Microfone", "Headset USB Logitech H390"];
let escolhido: string | null = null;

const estado = (): EstadoMicrofone => ({
  dispositivos: CONECTADOS,
  escolhido,
  conectado: escolhido === null || CONECTADOS.includes(escolhido),
});

/** Modo navegador (VITE_IPC=simulado): mesmo formato de `commands`, estado só em memória. */
export const simulado = {
  audioMicrofoneEstado: (): Resultado<EstadoMicrofone> =>
    Promise.resolve({ status: "ok", data: estado() }),
  audioDefinirMicrofone: (nome: string | null): Resultado<EstadoMicrofone> => {
    if (nome !== null && !CONECTADOS.includes(nome)) {
      return Promise.resolve({
        status: "error",
        error: {
          codigo: "microfone_desconectado",
          mensagem: "Este microfone não está conectado. Escolha um da lista.",
          campo: "microfone",
        },
      });
    }
    escolhido = nome;
    return Promise.resolve({ status: "ok", data: estado() });
  },
};

import type {
  EstadoMicrofone,
  ErroApp_Serialize,
  PreferenciaSons_Serialize,
} from "../../../ipc/bindings";

type Resultado<T> = Promise<
  { status: "ok"; data: T } | { status: "error"; error: ErroApp_Serialize }
>;

const CONECTADOS = ["MacBook Pro — Microfone", "Headset USB Logitech H390"];
let escolhido: string | null = null;
let pausarMidia = true;
let descartarSemFala = true;

const SONS = [
  { id: "gota", nome: "Gota" },
  { id: "pulso", nome: "Pulso" },
  { id: "clique_suave", nome: "Clique suave" },
];
let sons = { ligado: true, som: "gota" };
const prefSons = (): PreferenciaSons_Serialize => ({ ...sons, opcoes: SONS });

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
  midiaPausarObter: (): Resultado<boolean> => Promise.resolve({ status: "ok", data: pausarMidia }),
  midiaPausarDefinir: (ligado: boolean): Resultado<boolean> => {
    pausarMidia = ligado;
    return Promise.resolve({ status: "ok", data: ligado });
  },
  descartarSemFalaObter: (): Resultado<boolean> =>
    Promise.resolve({ status: "ok", data: descartarSemFala }),
  descartarSemFalaDefinir: (ligado: boolean): Resultado<boolean> => {
    descartarSemFala = ligado;
    return Promise.resolve({ status: "ok", data: ligado });
  },
  sonsObter: (): Resultado<PreferenciaSons_Serialize> =>
    Promise.resolve({ status: "ok", data: prefSons() }),
  sonsDefinir: (ligado: boolean, som: string): Resultado<PreferenciaSons_Serialize> => {
    if (!SONS.some((s) => s.id === som)) {
      return Promise.resolve({
        status: "error",
        error: {
          codigo: "som_invalido",
          mensagem: "Este som não existe. Escolha um da lista.",
          campo: "som",
        },
      });
    }
    sons = { ligado, som };
    return Promise.resolve({ status: "ok", data: prefSons() });
  },
  sonsPrevia: (_som: string): Resultado<null> => Promise.resolve({ status: "ok", data: null }),
};

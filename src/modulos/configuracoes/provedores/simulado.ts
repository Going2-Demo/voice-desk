/** Dados do modo navegador desta área: cofre e teste do Groq simulados, em memória. */
import type { Provedor } from "../../../ipc/bindings";
import type { ApiChaves, EstadoChave } from "./api";
import { ErroChave } from "./erro";

const URL_CHAVE = "https://console.groq.com/keys";
const vazio = (provedor: Provedor): EstadoChave => ({
  provedor,
  salva: false,
  mascara: null,
  validade: null,
  motivo: null,
});

let chaves: Record<string, { chave: string; estado: EstadoChave }> = {};

const pausa = (ms: number) => new Promise((r) => setTimeout(r, ms));

/** No simulado, uma chave que contém "invalida" é recusada; qualquer outra é aceita. */
function testar(chave: string): Pick<EstadoChave, "validade" | "motivo"> {
  return /invalida/i.test(chave)
    ? { validade: "invalida", motivo: "Invalid API Key" }
    : { validade: "valida", motivo: null };
}

export const simulado: ApiChaves = {
  async estado() {
    return [chaves.groq?.estado ?? vazio("groq")];
  },
  async salvar(provedor, bruta) {
    const chave = bruta.trim();
    if (!chave) throw new ErroChave("chave_vazia", "Cole a chave de API do provedor.", "chave");
    await pausa(900);
    const estado: EstadoChave = {
      provedor,
      salva: true,
      mascara: `••••${chave.slice(-4)}`,
      ...testar(chave),
    };
    chaves[provedor] = { chave, estado };
    return estado;
  },
  async testar(provedor) {
    const atual = chaves[provedor];
    if (!atual) throw new ErroChave("chave_ausente", "Cadastre a chave do Groq.");
    await pausa(500);
    atual.estado = { ...atual.estado, ...testar(atual.chave) };
    return atual.estado;
  },
  async apagar(provedor) {
    delete chaves[provedor];
    return vazio(provedor);
  },
  async abrirPagina() {
    window.open(URL_CHAVE, "_blank", "noopener");
  },
};

/** Só para os testes: volta ao estado inicial (sem chave). */
export function reiniciarSimulado() {
  chaves = {};
}

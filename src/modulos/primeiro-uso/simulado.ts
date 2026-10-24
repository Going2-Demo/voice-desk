/** Modo navegador: o primeiro uso vive em memória e o ditado de teste é simulado. */
import type { ApiPrimeiroUso, EstadoPrimeiroUso } from "./api";

let estado: EstadoPrimeiroUso = { concluido: false, pulados: [] };
const ORDEM = ["permissoes", "atalho", "provedor", "sincronizacao"];

const copia = () => ({ ...estado, pulados: [...estado.pulados] });

export const simulado: ApiPrimeiroUso = {
  async estado() {
    return copia();
  },
  async pular(passo) {
    if (!estado.pulados.includes(passo)) {
      estado.pulados = ORDEM.filter((p) => p === passo || estado.pulados.includes(p));
    }
    return copia();
  },
  async cumprir(passo) {
    estado = { ...estado, pulados: estado.pulados.filter((p) => p !== passo) };
    return copia();
  },
  async concluir() {
    estado = { ...estado, concluido: true };
    return copia();
  },
  simularDitado: () => "Funcionou! Este é o meu primeiro ditado com o Voice Desk.",
};

/** Modo navegador: permissões em memória. `?plat=mac` mostra a variante do Mac (padrão: Windows). */
import type { ApiPermissoes, EstadoPermissoes } from "./api";

const mac = () => new URLSearchParams(window.location.search).get("plat") === "mac";

let estado: EstadoPermissoes | undefined;
const atual = (): EstadoPermissoes =>
  (estado ??= {
    plataforma: mac() ? "macos" : "windows",
    microfone: "pendente",
    acessibilidade: mac() ? "pendente" : "nao_se_aplica",
  });

export const simulado: ApiPermissoes = {
  async estado() {
    return { ...atual() };
  },
  async pedir(permissao) {
    const e = atual();
    estado = { ...e, [permissao]: "concedida" };
    return { ...estado };
  },
  async abrirConfiguracoes() {},
};

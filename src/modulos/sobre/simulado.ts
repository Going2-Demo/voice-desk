import type { ApiSobre, Sobre } from "./api";

const URL_PROJETO = "https://going2-demo.github.io/voice-desk/";

const dados: Sobre = {
  versao: "0.1.0",
  commit: "local",
  licenca: "MIT",
  url_projeto: URL_PROJETO,
};

export const simulado: ApiSobre = {
  async obter() {
    return dados;
  },
  async abrirProjeto() {
    window.open(URL_PROJETO, "_blank", "noopener");
  },
};

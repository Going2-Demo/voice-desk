import { type Atalho, commands, type EstadoGravador_Deserialize, events } from "./bindings";

export type Estado = EstadoGravador_Deserialize;
export type ResultadoAtalho = { ok: true; atalho: Atalho } | { ok: false; mensagem: string };
type Cancelar = () => void;

/** Tudo que as telas do ditado pedem ao núcleo; em teste e no modo simulado entra uma troca. */
export type PonteDitado = {
  obterAtalho: () => Promise<Atalho>;
  salvarAtalho: (combinacao: string) => Promise<ResultadoAtalho>;
  alternarDitado: () => Promise<void>;
  aoEstado: (ouvinte: (e: Estado) => void) => Promise<Cancelar>;
  aoNivel: (ouvinte: (nivel: number) => void) => Promise<Cancelar>;
};

export const ponteReal: PonteDitado = {
  async obterAtalho() {
    const r = await commands.atalhosObter();
    if (r.status === "error") throw new Error(r.error.mensagem);
    return r.data;
  },
  async salvarAtalho(combinacao) {
    const r = await commands.atalhosSalvar(combinacao);
    return r.status === "ok"
      ? { ok: true, atalho: r.data }
      : { ok: false, mensagem: r.error.mensagem };
  },
  async alternarDitado() {
    await commands.ditadoAlternar();
  },
  aoEstado: (ouvinte) => events.ditadoEstado.listen((e) => ouvinte(e.payload)),
  aoNivel: (ouvinte) => events.ditadoNivel.listen((e) => ouvinte(e.payload ?? 0)),
};

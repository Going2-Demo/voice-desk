import { commands } from "../../../ipc/bindings";
import { simulado } from "./simulado";

/** Comandos da área: os reais, ou os simulados quando a interface roda no navegador. */
export const api = import.meta.env.VITE_IPC === "simulado" ? simulado : commands;

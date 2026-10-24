/** Liga o simulado.ts de cada área ao modo navegador (VITE_IPC=simulado). */
import { simulado as bandeja } from "../modulos/bandeja/simulado";
import { simulado as dados } from "../modulos/configuracoes/dados/simulado";
import { simulado as geral } from "../modulos/configuracoes/geral/simulado";
import { simulado as atalhos } from "../modulos/configuracoes/atalhos/simulado";
import { simulado as modeloLocal } from "../modulos/configuracoes/modelo-local/simulado";
import { simulado as permissoes } from "../modulos/configuracoes/permissoes/simulado";
import { simulado as provedores } from "../modulos/configuracoes/provedores/simulado";
import { simulado as historico } from "../modulos/historico/simulado";
import { simulado as limpeza } from "../modulos/limpeza/simulado";
import { simulado as modos } from "../modulos/modos/simulado";
import { simulado as primeiroUso } from "../modulos/primeiro-uso/simulado";
import { simulado as sincronizacao } from "../modulos/sincronizacao/simulado";
import { simulado as sobre } from "../modulos/sobre/simulado";
import { simulado as substituicoes } from "../modulos/substituicoes/simulado";
import { simulado as vocabulario } from "../modulos/vocabulario/simulado";

export const simulados = {
  bandeja,
  dados,
  geral,
  atalhos,
  modeloLocal,
  permissoes,
  provedores,
  historico,
  limpeza,
  modos,
  primeiroUso,
  sincronizacao,
  sobre,
  substituicoes,
  vocabulario,
};

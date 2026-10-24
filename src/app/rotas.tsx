import { type ComponentType, lazy } from "react";

export type Rota = {
  caminho: string;
  rotulo: string;
  grupo: "Ditado" | "Texto" | "Máquinas" | "App";
  /** Item da lateral; rotas com parâmetro (`:id`) não aparecem nela. */
  naLateral: boolean;
  Pagina: ComponentType;
};

const pagina = (carregar: () => Promise<{ default: ComponentType }>) => lazy(carregar);

/** Todas as rotas do design 2.2; cada história troca só o conteúdo da própria página. */
export const rotas: Rota[] = [
  {
    caminho: "/modos",
    rotulo: "Modos",
    grupo: "Ditado",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/modos/pagina")),
  },
  {
    caminho: "/modos/:id",
    rotulo: "Editor de modo",
    grupo: "Ditado",
    naLateral: false,
    Pagina: pagina(() => import("../modulos/modos/editor/pagina")),
  },
  {
    caminho: "/historico",
    rotulo: "Histórico",
    grupo: "Ditado",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/historico/pagina")),
  },
  {
    caminho: "/historico/:id",
    rotulo: "Item do histórico",
    grupo: "Ditado",
    naLateral: false,
    Pagina: pagina(() => import("../modulos/historico/detalhe")),
  },
  {
    caminho: "/vocabulario",
    rotulo: "Vocabulário",
    grupo: "Texto",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/vocabulario/pagina")),
  },
  {
    caminho: "/substituicoes",
    rotulo: "Substituições",
    grupo: "Texto",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/substituicoes/pagina")),
  },
  {
    caminho: "/limpeza",
    rotulo: "Limpeza do texto",
    grupo: "Texto",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/limpeza/pagina")),
  },
  {
    caminho: "/sincronizacao",
    rotulo: "Sincronização",
    grupo: "Máquinas",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/sincronizacao/pagina")),
  },
  {
    caminho: "/config/geral",
    rotulo: "Geral",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/configuracoes/geral/pagina")),
  },
  {
    caminho: "/config/atalhos",
    rotulo: "Atalhos",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/configuracoes/atalhos/pagina")),
  },
  {
    caminho: "/config/provedores",
    rotulo: "Provedores e chaves",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/configuracoes/provedores/pagina")),
  },
  {
    caminho: "/config/local",
    rotulo: "Modelo local",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/configuracoes/modelo-local/pagina")),
  },
  {
    caminho: "/config/dados",
    rotulo: "Dados e retenção",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/configuracoes/dados/pagina")),
  },
  {
    caminho: "/config/permissoes",
    rotulo: "Permissões",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/configuracoes/permissoes/pagina")),
  },
  {
    caminho: "/bandeja",
    rotulo: "Ícone e menu",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/bandeja/pagina")),
  },
  {
    caminho: "/primeiro-uso",
    rotulo: "Primeiro uso",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/primeiro-uso/pagina")),
  },
  {
    caminho: "/sobre",
    rotulo: "Sobre",
    grupo: "App",
    naLateral: true,
    Pagina: pagina(() => import("../modulos/sobre/pagina")),
  },
];

export const rotaInicial = "/modos";

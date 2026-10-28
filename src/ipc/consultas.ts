/** Chaves do TanStack Query por área. Cada história acrescenta as suas. */
export const consultas = {
  app: { sobre: ["app", "sobre"] as const },
  chaves: { estado: ["chaves", "estado"] as const },
  atalhos: { global: ["atalhos", "global"] as const },
  permissoes: { estado: ["permissoes", "estado"] as const },
  vocabulario: { lista: ["vocabulario", "lista"] as const },
  substituicoes: { lista: ["substituicoes", "lista"] as const },
  limpeza: {
    lista: ["limpeza", "lista"] as const,
    opcoes: ["limpeza", "opcoes"] as const,
    previa: ["limpeza", "previa"] as const,
  },
  primeiroUso: { estado: ["primeiro-uso", "estado"] as const },
};

/**
 * Espelho em TypeScript de `src-tauri/src/limpeza/algoritmo.rs`, só para o modo navegador
 * (`VITE_IPC=simulado`). No app, quem limpa o texto é o núcleo em Rust.
 */
type Entrada = { palavras: string[]; exigeReticencias: boolean };
type Ficha = { inteira: string; nucleo: string; sufixo: string };

export const normalizarEntrada = (p: string) =>
  p.replaceAll("...", "…").split(/\s+/).filter(Boolean).join(" ").toLowerCase();

function entrada(palavra: string): Entrada | null {
  const norm = normalizarEntrada(palavra);
  const exigeReticencias = norm.endsWith("…");
  const palavras = norm.replace(/…+$/u, "").split(/\s+/).filter(Boolean);
  return palavras.length ? { palavras, exigeReticencias } : null;
}

const ehAlfaNum = (c: string) => /[\p{L}\p{N}]/u.test(c);

function fichas(texto: string): Ficha[] {
  return texto
    .split(/\s+/)
    .filter(Boolean)
    .map((inteira) => {
      const cs = [...inteira];
      let fim = 0;
      for (let i = cs.length - 1; i >= 0; i--) {
        if (ehAlfaNum(cs[i])) {
          fim = i + 1;
          break;
        }
      }
      const miolo = cs.slice(0, fim);
      const inicio = miolo.findIndex(ehAlfaNum);
      const nucleo = (inicio < 0 ? "" : miolo.slice(inicio).join("")).toLowerCase();
      return { inteira, nucleo, sufixo: fim === 0 ? inteira : cs.slice(fim).join("") };
    });
}

function casa(e: Entrada, fs: Ficha[]): boolean {
  const n = e.palavras.length;
  if (fs.length < n) return false;
  const janela = fs.slice(0, n);
  return (
    janela.every((f, i) => f.nucleo === e.palavras[i]) &&
    janela.slice(0, n - 1).every((f) => f.sufixo === "") &&
    (!e.exigeReticencias || /…|\.\.\./.test(janela[n - 1].sufixo))
  );
}

export function removerPreenchimento(texto: string, lista: string[]): string {
  const entradas = lista
    .map(entrada)
    .filter((e): e is Entrada => e !== null)
    .sort((a, b) => b.palavras.length - a.palavras.length);
  const todas = fichas(texto);
  const saida: string[] = [];
  let i = 0;
  while (i < todas.length) {
    const resto = todas.slice(i);
    const e = entradas.find((x) => casa(x, resto));
    if (!e) {
      saida.push(todas[i].inteira);
      i += 1;
      continue;
    }
    const marcas = [...resto[e.palavras.length - 1].sufixo].filter((c) => c === "?" || c === "!");
    if (marcas.length && saida.length) {
      saida[saida.length - 1] = saida[saida.length - 1].replace(/[,;]+$/, "") + marcas.join("");
    }
    i += e.palavras.length;
  }
  return saida.join(" ");
}

export function formatar(texto: string): string {
  const compacto = texto.split(/\s+/).filter(Boolean).join(" ");
  const sobra = compacto.replace(/^[,;]+/, "").replace(/[,;]+$/, "");
  if (!sobra) return "";
  const [primeira, ...resto] = [...sobra];
  const saida = primeira.toUpperCase() + resto.join("");
  return /[.!?…:)"”»]$/.test(saida) ? saida : `${saida}.`;
}

export function limpar(texto: string, lista: string[], remover: boolean, formatarTexto: boolean) {
  let t = texto;
  if (remover) t = removerPreenchimento(t, lista);
  if (formatarTexto) t = formatar(t);
  return t;
}

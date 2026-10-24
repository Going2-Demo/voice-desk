import type { Page } from "@playwright/test";

type Resposta = unknown | ((args: Record<string, unknown>) => unknown);

/** Troca o IPC do Tauri no navegador: comandos respondem o que o teste definir e eventos são emitidos por `emitir`. */
export async function instalarTauri(page: Page, comandos: Record<string, Resposta>) {
  await page.exposeFunction("__chamar", (cmd: string, args: Record<string, unknown>) => {
    const r = comandos[cmd];
    if (r === undefined) throw new Error(`comando sem resposta: ${cmd}`);
    return typeof r === "function" ? (r as (a: unknown) => unknown)(args) : r;
  });
  await page.addInitScript(() => {
    const w = window as unknown as Record<string, unknown>;
    const callbacks = new Map<number, (e: unknown) => void>();
    const ouvintes = new Map<string, number[]>();
    let id = 1;
    w.__TAURI_INTERNALS__ = {
      transformCallback(cb: (e: unknown) => void) {
        const n = id++;
        callbacks.set(n, cb);
        return n;
      },
      unregisterCallback(n: number) {
        callbacks.delete(n);
      },
      convertFileSrc: (s: string) => s,
      invoke(cmd: string, args: Record<string, unknown> = {}) {
        if (cmd === "plugin:event|listen") {
          const lista = ouvintes.get(args.event as string) ?? [];
          lista.push(args.handler as number);
          ouvintes.set(args.event as string, lista);
          return Promise.resolve(id++);
        }
        if (cmd.startsWith("plugin:event|")) return Promise.resolve();
        return (w.__chamar as (c: string, a: unknown) => Promise<unknown>)(cmd, args);
      },
    };
    w.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener() {} };
    w.__emitir = (evento: string, payload: unknown) => {
      for (const h of ouvintes.get(evento) ?? [])
        callbacks.get(h)?.({ event: evento, id: 0, payload });
    };
  });
}

export async function emitir(page: Page, evento: string, payload: unknown) {
  await page.evaluate(
    ([e, p]) =>
      (window as unknown as { __emitir: (a: string, b: unknown) => void }).__emitir(e as string, p),
    [evento, payload],
  );
}

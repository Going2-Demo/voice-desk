import { expect, test } from "@playwright/test";
import { emitir, instalarTauri } from "../ditado/tauri-mock";

const estado = (fase: string, mensagem: string) => ({ fase, mensagem, segundos: 0 });

test.describe("ENG-68 · F-03 colar no programa em foco (interface, núcleo simulado)", () => {
  test("RF-03.4 não colou: o gravador mostra o aviso com o atalho de colar manual (Mac)", async ({
    page,
  }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", estado("processando", "Processando…"));
    await emitir(page, "ditado-estado", estado("aviso", "Texto copiado; cole com Cmd+V"));
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "aviso");
    await expect(page.getByRole("status")).toHaveText("Texto copiado; cole com Cmd+V");
  });

  test("RF-03.4 não colou: o gravador mostra o aviso com o atalho de colar manual (Windows)", async ({
    page,
  }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", estado("aviso", "Texto copiado; cole com Ctrl+V"));
    await expect(page.getByRole("status")).toHaveText("Texto copiado; cole com Ctrl+V");
    await expect(page.getByRole("meter")).toHaveCount(0);
  });

  test("colagem concluída: o gravador termina em 'ok' sem aviso de texto copiado", async ({
    page,
  }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", estado("processando", "Processando…"));
    await emitir(page, "ditado-estado", estado("ok", "Pronto"));
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "ok");
    await expect(page.getByText(/texto copiado/i)).toHaveCount(0);
  });
});

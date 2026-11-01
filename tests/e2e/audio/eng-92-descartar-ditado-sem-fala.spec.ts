import { expect, test } from "@playwright/test";
import { irPara } from "../helpers/navegar";
import { emitir, instalarTauri } from "../ditado/tauri-mock";

const voltarParaGeral = (page: import("@playwright/test").Page) =>
  page.evaluate(async () => {
    window.location.hash = "#/modos";
    await new Promise((r) => setTimeout(r, 100));
    window.location.hash = "#/config/geral";
  });

test.describe("ENG-92 · F-26 descartar ditado sem fala (interface, núcleo simulado)", () => {
  test("RF-26.1 seção 'Ditado sem fala': descarte ligado por padrão", async ({ page }) => {
    await irPara(page, "/config/geral");
    await expect(page.getByRole("heading", { level: 2, name: "Ditado sem fala" })).toBeVisible();
    await expect(page.getByRole("switch", { name: "Descartar ditado sem fala" })).toBeChecked();
    await expect(page.getByText(/o áudio não é enviado ao provedor e nada é colado/)).toBeVisible();
  });

  test("RF-26.1 desligar e religar salva a preferência e ela persiste ao reabrir a tela", async ({
    page,
  }) => {
    await irPara(page, "/config/geral");
    const interruptor = page.getByRole("switch", { name: "Descartar ditado sem fala" });
    await interruptor.setChecked(false, { force: true });
    await expect(page.getByText("Preferência salva")).toBeVisible();
    await expect(interruptor).not.toBeChecked();
    await voltarParaGeral(page);
    await expect(page.getByRole("switch", { name: "Descartar ditado sem fala" })).not.toBeChecked();

    await page
      .getByRole("switch", { name: "Descartar ditado sem fala" })
      .setChecked(true, { force: true });
    await expect(page.getByRole("switch", { name: "Descartar ditado sem fala" })).toBeChecked();
    await voltarParaGeral(page);
    await expect(page.getByRole("switch", { name: "Descartar ditado sem fala" })).toBeChecked();
    await expect(page.getByRole("alert")).toHaveCount(0);
  });

  test("RF-26.2 gravação sem fala: gravador mostra 'Nenhuma fala detectada.' como aviso", async ({
    page,
  }) => {
    await instalarTauri(page, {});
    await page.goto("/gravador.html");
    await emitir(page, "ditado-estado", { fase: "gravando", mensagem: "Gravando…", segundos: 5 });
    await emitir(page, "ditado-estado", {
      fase: "processando",
      mensagem: "Processando…",
      segundos: 0,
    });
    await emitir(page, "ditado-estado", {
      fase: "aviso",
      mensagem: "Nenhuma fala detectada.",
      segundos: 0,
    });
    await expect(page.locator(".gravador")).toHaveAttribute("data-fase", "aviso");
    await expect(page.getByRole("status")).toHaveText("Nenhuma fala detectada.");
  });
});

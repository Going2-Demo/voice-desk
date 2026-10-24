import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import type { PonteDitado } from "../../../ipc/ponte-ditado";
import { PaginaAtalhos } from "./pagina";

const atual = {
  combinacao: "Ctrl+Alt+Space",
  exibicao: "Ctrl+Alt+Espaço",
  padrao: "Ctrl+Alt+Space",
};

function desenhar(salvar: PonteDitado["salvarAtalho"]) {
  const ponte = {
    obterAtalho: vi.fn().mockResolvedValue(atual),
    salvarAtalho: vi.fn(salvar),
  } as unknown as PonteDitado;
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <PaginaAtalhos ponte={ponte} />
    </QueryClientProvider>,
  );
  return ponte;
}

describe("Configurações · Atalhos", () => {
  it("mostra o atalho atual e a dica do Esc duplo", async () => {
    desenhar(async () => ({ ok: true, atalho: atual }));
    expect(await screen.findByLabelText("Atalho")).toHaveValue("Ctrl+Alt+Espaço");
    expect(screen.getByText(/duas vezes em até 1,5 s/)).toBeInTheDocument();
  });

  it("salva um atalho válido e avisa que já vale sem reiniciar", async () => {
    const novo = { combinacao: "Ctrl+Alt+D", exibicao: "Ctrl+Alt+D", padrao: atual.padrao };
    const ponte = desenhar(async () => ({ ok: true, atalho: novo }));
    const campo = await screen.findByLabelText("Atalho");
    await userEvent.clear(campo);
    await userEvent.type(campo, "Ctrl+Alt+D");
    await userEvent.click(screen.getByRole("button", { name: "Salvar atalho" }));
    expect(ponte.salvarAtalho).toHaveBeenCalledWith("Ctrl+Alt+D");
    expect(await screen.findByRole("status")).toHaveTextContent(
      "Atalho salvo. Já vale, sem reiniciar o app.",
    );
    expect(campo).toHaveValue("Ctrl+Alt+D");
  });

  it("mostra a recusa do núcleo no campo, com aria-invalid", async () => {
    desenhar(async () => ({
      ok: false,
      mensagem: "O atalho precisa de pelo menos uma tecla modificadora (Ctrl, Alt, Shift ou ⌘).",
    }));
    const campo = await screen.findByLabelText("Atalho");
    await userEvent.clear(campo);
    await userEvent.type(campo, "A");
    await userEvent.click(screen.getByRole("button", { name: "Salvar atalho" }));
    const alerta = await screen.findByRole("alert");
    expect(alerta).toHaveTextContent("modificadora");
    await waitFor(() => expect(campo).toHaveAttribute("aria-invalid", "true"));
    expect(screen.queryByRole("status")).toBeNull();
  });

  it("não envia campo vazio", async () => {
    const ponte = desenhar(async () => ({ ok: true, atalho: atual }));
    const campo = await screen.findByLabelText("Atalho");
    await userEvent.clear(campo);
    await userEvent.click(screen.getByRole("button", { name: "Salvar atalho" }));
    expect(await screen.findByRole("alert")).toHaveTextContent("Digite");
    expect(ponte.salvarAtalho).not.toHaveBeenCalled();
  });
});

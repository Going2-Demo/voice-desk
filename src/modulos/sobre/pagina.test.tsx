import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import type { ApiSobre } from "./api";

const fake = vi.hoisted(() => ({ obter: vi.fn(), abrirProjeto: vi.fn() }));
vi.mock("./api", () => ({ api: fake satisfies ApiSobre }));

import Pagina from "./pagina";

function desenhar() {
  const cliente = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={cliente}>
      <Pagina />
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  fake.obter.mockReset();
  fake.abrirProjeto.mockReset();
  fake.obter.mockResolvedValue({
    versao: "0.1.0",
    commit: "abc1234",
    licenca: "MIT",
    url_projeto: "https://going2-demo.github.io/voice-desk/",
  });
  fake.abrirProjeto.mockResolvedValue(undefined);
});

describe("Sobre (RF-27.4)", () => {
  it("mostra versão instalada, licença e o aviso de que não tem relação com o VoiceInk", async () => {
    desenhar();
    expect(await screen.findByText("0.1.0")).toBeInTheDocument();
    expect(screen.getByText("MIT")).toBeInTheDocument();
    expect(screen.getByText(/não tem relação com o VoiceInk/)).toBeInTheDocument();
  });

  it("o link da página do projeto abre o navegador padrão pelo núcleo", async () => {
    desenhar();
    await screen.findByText("0.1.0");
    await userEvent.click(screen.getByRole("button", { name: /página do projeto/i }));
    expect(fake.abrirProjeto).toHaveBeenCalledTimes(1);
  });

  it("avisa quando não consegue abrir a página", async () => {
    fake.abrirProjeto.mockRejectedValue(new Error("sem navegador"));
    desenhar();
    await screen.findByText("0.1.0");
    await userEvent.click(screen.getByRole("button", { name: /página do projeto/i }));
    expect(await screen.findByRole("alert")).toHaveTextContent(/não foi possível abrir/i);
  });

  it("mostra erro se a versão não carrega", async () => {
    fake.obter.mockRejectedValue(new Error("falhou"));
    desenhar();
    expect(await screen.findByRole("alert")).toHaveTextContent(/não foi possível ler/i);
  });
});

import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { describe, expect, it, vi } from "vitest";
import type { PonteDitado } from "../../ipc/ponte-ditado";
import { PaginaInicio } from "./pagina";

function desenhar(ponte: Partial<PonteDitado>) {
  const cliente = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={cliente}>
      <MemoryRouter>
        <PaginaInicio ponte={ponte as PonteDitado} />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("Início", () => {
  it("explica como ditar com o atalho atual", async () => {
    desenhar({ obterAtalho: vi.fn().mockResolvedValue({ exibicao: "Ctrl+Alt+Espaço" }) });
    expect(await screen.findByText("Ctrl+Alt+Espaço")).toBeInTheDocument();
    expect(screen.getByRole("heading", { name: "Como ditar" })).toBeInTheDocument();
  });

  it("leva ao guia de primeiro uso", async () => {
    desenhar({ obterAtalho: vi.fn().mockResolvedValue({ exibicao: "X" }) });
    expect(screen.getByRole("link", { name: "Guia de primeiro uso" })).toHaveAttribute(
      "href",
      "/primeiro-uso",
    );
  });

  it("avisa quando não lê o atalho", async () => {
    desenhar({ obterAtalho: vi.fn().mockRejectedValue(new Error("x")) });
    expect(await screen.findByRole("alert")).toHaveTextContent("Não foi possível ler o atalho");
  });

  it("dispara o ditado de teste", async () => {
    const alternar = vi.fn().mockResolvedValue(undefined);
    desenhar({
      obterAtalho: vi.fn().mockResolvedValue({ exibicao: "X" }),
      alternarDitado: alternar,
    });
    await userEvent.click(screen.getByRole("button", { name: "Testar ditado" }));
    expect(alternar).toHaveBeenCalled();
  });
});

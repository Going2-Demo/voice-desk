import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen } from "@testing-library/react";
import { MemoryRouter } from "react-router";
import { describe, expect, it } from "vitest";
import { App } from "./app";
import { rotas } from "./rotas";

const desenhar = (caminho: string) =>
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <MemoryRouter initialEntries={[caminho]}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>,
  );

describe("rotas da janela principal", () => {
  it("cria todas as rotas do design 2.2", () => {
    const caminhos = rotas.map((r) => r.caminho).sort();
    expect(caminhos).toEqual(
      [
        "/inicio",
        "/modos",
        "/modos/:id",
        "/historico",
        "/historico/:id",
        "/vocabulario",
        "/substituicoes",
        "/limpeza",
        "/sincronizacao",
        "/config/geral",
        "/config/atalhos",
        "/config/provedores",
        "/config/local",
        "/config/dados",
        "/config/permissoes",
        "/bandeja",
        "/primeiro-uso",
        "/sobre",
      ].sort(),
    );
  });

  it("lista na lateral só as rotas sem parâmetro", () => {
    desenhar("/modos");
    expect(screen.getByRole("link", { name: "Início" })).toHaveAttribute("href", "/inicio");
    expect(screen.getByRole("link", { name: "Vocabulário" })).toHaveAttribute(
      "href",
      "/vocabulario",
    );
    expect(screen.queryByRole("link", { name: "Editor de modo" })).toBeNull();
  });

  it("mostra a página provisória da rota pedida", async () => {
    desenhar("/historico/101");
    expect(
      await screen.findByRole("heading", { name: "Histórico · item 101" }),
    ).toBeInTheDocument();
  });

  it("volta para o Início numa rota desconhecida", async () => {
    desenhar("/nao-existe");
    expect(await screen.findByRole("heading", { name: "Início" })).toBeInTheDocument();
  });
});

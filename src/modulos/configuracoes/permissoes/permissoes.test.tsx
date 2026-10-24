import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import type { ApiPermissoes, EstadoPermissoes } from "./api";
import { ListaPermissoes } from "./permissoes";

const base: EstadoPermissoes = {
  plataforma: "windows",
  microfone: "concedida",
  acessibilidade: "nao_se_aplica",
};

function api(estado: EstadoPermissoes, extra: Partial<ApiPermissoes> = {}): ApiPermissoes {
  return {
    estado: vi.fn().mockResolvedValue(estado),
    pedir: vi.fn().mockResolvedValue(estado),
    abrirConfiguracoes: vi.fn().mockResolvedValue(undefined),
    ...extra,
  };
}

const desenhar = (a: ApiPermissoes) =>
  render(
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <ListaPermissoes api={a} />
    </QueryClientProvider>,
  );

describe("permissões (RF-06.1)", () => {
  it("explica para que serve e mostra que o microfone foi concedida", async () => {
    desenhar(api(base));
    expect(await screen.findByText("Para gravar a sua voz.")).toBeInTheDocument();
    expect(screen.getByRole("status", { name: "Microfone: Concedida" })).toBeInTheDocument();
    expect(screen.queryByRole("button", { name: /conceder/i })).toBeNull();
  });

  it("microfone negado mostra Negada, o que deixa de funcionar e como conceder", async () => {
    desenhar(api({ ...base, microfone: "negada" }));
    expect(await screen.findByRole("status", { name: "Microfone: Negada" })).toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent("Sem o microfone o ditado não grava nada.");
    expect(screen.getByRole("note")).toHaveTextContent(/Configurações do Windows/);
  });

  it("abre a tela do sistema para a permissão negada", async () => {
    const a = api({ ...base, microfone: "negada" });
    desenhar(a);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir configurações do sistema" }));
    await waitFor(() => expect(a.abrirConfiguracoes).toHaveBeenCalledWith("microfone"));
  });

  it("conceder pede ao sistema e mostra o estado novo", async () => {
    const a = api(
      { ...base, microfone: "pendente" },
      { pedir: vi.fn().mockResolvedValue({ ...base, microfone: "concedida" }) },
    );
    desenhar(a);
    fireEvent.click(await screen.findByRole("button", { name: "Conceder microfone" }));
    await waitFor(() => expect(a.pedir).toHaveBeenCalledWith("microfone"));
    expect(await screen.findByRole("status", { name: "Microfone: Concedida" })).toBeInTheDocument();
  });

  it("no Mac lista também a Acessibilidade, com o aviso do que deixa de funcionar", async () => {
    desenhar(api({ plataforma: "macos", microfone: "concedida", acessibilidade: "negada" }));
    expect(
      await screen.findByRole("status", { name: "Acessibilidade: Negada" }),
    ).toBeInTheDocument();
    expect(screen.getByRole("note")).toHaveTextContent(/Cmd\+V/);
    expect(screen.getByRole("note")).toHaveTextContent(/Ajustes do Sistema/);
  });

  it("no Windows não lista Acessibilidade", async () => {
    desenhar(api(base));
    await screen.findByText("Microfone");
    expect(screen.queryByText("Acessibilidade")).toBeNull();
  });

  it("mostra o erro do sistema quando não consegue abrir as configurações", async () => {
    const a = api(
      { ...base, microfone: "negada" },
      { abrirConfiguracoes: vi.fn().mockRejectedValue(new Error("Não abriu.")) },
    );
    desenhar(a);
    fireEvent.click(await screen.findByRole("button", { name: "Abrir configurações do sistema" }));
    await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Não abriu."));
  });
});

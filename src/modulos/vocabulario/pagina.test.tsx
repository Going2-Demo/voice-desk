import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
vi.hoisted(() => {
  import.meta.env.VITE_IPC = "simulado";
});

import { CaixaRapida } from "../../janelas/menu/caixa-rapida";
import { reiniciarSimulado } from "./simulado";
import Pagina from "./pagina";

function desenhar() {
  const cliente = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={cliente}>
      <Pagina />
    </QueryClientProvider>,
  );
}

beforeEach(() => reiniciarSimulado(["Going2", "Kubernetes"]));

describe("Vocabulário (F-07)", () => {
  it("mostra as palavras e o contador", async () => {
    desenhar();
    expect(await screen.findByText("Going2")).toBeInTheDocument();
    expect(screen.getByText("2 de 500 palavras")).toBeInTheDocument();
  });

  it("palavra adicionada aparece na lista e na busca", async () => {
    desenhar();
    await userEvent.type(await screen.findByLabelText("Adicionar palavra"), "Hortolândia");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    const lista = await screen.findByRole("list", { name: "Palavras do vocabulário" });
    expect(await within(lista).findByText("Hortolândia")).toBeInTheDocument();
    await userEvent.type(screen.getByLabelText("Buscar no vocabulário"), "hortol");
    await waitFor(() => expect(within(lista).queryByText("Going2")).not.toBeInTheDocument());
    expect(within(lista).getByText("Hortolândia")).toBeInTheDocument();
  });

  it("duplicada sem diferenciar maiúsculas é recusada com aviso", async () => {
    desenhar();
    await userEvent.type(await screen.findByLabelText("Adicionar palavra"), "going2");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(await screen.findByText("“going2” já está no vocabulário.")).toBeInTheDocument();
  });

  it("Enter no campo envia e mostra a duplicata", async () => {
    desenhar();
    await userEvent.type(await screen.findByLabelText("Adicionar palavra"), "going2{Enter}");
    expect(await screen.findByText("“going2” já está no vocabulário.")).toBeInTheDocument();
    expect(screen.getByLabelText("Adicionar palavra")).toHaveAttribute("aria-invalid", "true");
  });

  it("avisa quando só parte das palavras vai como dica", async () => {
    reiniciarSimulado(Array.from({ length: 100 }, (_, i) => `termo-numero-${i}`));
    desenhar();
    expect(await screen.findByText(/palavras mais recentes vão como dica/)).toBeInTheDocument();
  });

  it("campo vazio pede uma palavra", async () => {
    desenhar();
    await screen.findByText("Going2");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(screen.getByText("Digite uma palavra ou nome.")).toBeInTheDocument();
  });

  it("a 501ª palavra é recusada com a mensagem do limite", async () => {
    reiniciarSimulado(Array.from({ length: 500 }, (_, i) => `p${i}`));
    desenhar();
    await userEvent.type(await screen.findByLabelText("Adicionar palavra"), "mais uma");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(await screen.findByText("Limite de 500 palavras.")).toBeInTheDocument();
  });

  it("edita uma palavra", async () => {
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Editar Kubernetes" }));
    const campo = screen.getByLabelText("Palavra");
    await userEvent.clear(campo);
    await userEvent.type(campo, "K8s");
    await userEvent.click(screen.getByRole("button", { name: "Salvar" }));
    expect(await screen.findByText("K8s")).toBeInTheDocument();
    expect(screen.queryByText("Kubernetes")).not.toBeInTheDocument();
  });

  it("remove uma palavra", async () => {
    desenhar();
    await userEvent.click(await screen.findByRole("button", { name: "Remover Going2" }));
    await waitFor(() => expect(screen.queryByText("Going2")).not.toBeInTheDocument());
    expect(screen.getByText("1 de 500 palavras")).toBeInTheDocument();
  });

  it("vocabulário vazio mostra o estado vazio com o campo da primeira palavra", async () => {
    reiniciarSimulado([]);
    desenhar();
    expect(await screen.findByText("Seu vocabulário está vazio")).toBeInTheDocument();
    expect(screen.getByLabelText("Primeira palavra")).toBeInTheDocument();
  });

  it("busca sem resultado oferece limpar", async () => {
    desenhar();
    await userEvent.type(await screen.findByLabelText("Buscar no vocabulário"), "zzz");
    expect(await screen.findByText(/Nenhuma palavra encontrada/)).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Limpar busca" }));
    expect(await screen.findByText("Going2")).toBeInTheDocument();
  });
});

describe("Caixa rápida do menu (RF-07.3)", () => {
  it("a palavra digitada na caixa rápida aparece na lista do vocabulário", async () => {
    render(<CaixaRapida />);
    await userEvent.type(screen.getByLabelText("Adicionar ao vocabulário"), "Dokploy");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(await screen.findByText("“Dokploy” adicionada ao vocabulário.")).toBeInTheDocument();
    desenhar();
    expect(await screen.findByText("Dokploy")).toBeInTheDocument();
  });

  it("recusa duplicada e vazia com aviso", async () => {
    render(<CaixaRapida />);
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(screen.getByRole("alert")).toHaveTextContent("Digite uma palavra ou nome.");
    await userEvent.type(screen.getByLabelText("Adicionar ao vocabulário"), "GOING2");
    await userEvent.click(screen.getByRole("button", { name: "Adicionar" }));
    expect(await screen.findByText("“GOING2” já está no vocabulário.")).toBeInTheDocument();
  });
});

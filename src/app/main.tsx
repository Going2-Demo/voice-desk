import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { HashRouter } from "react-router";
import "../compartilhado/tema/tokens.css";
import "../compartilhado/tema/base.css";
import { App } from "./app";

const consultas = new QueryClient();

const raiz = document.getElementById("raiz");
if (!raiz) throw new Error("Elemento #raiz não encontrado");

createRoot(raiz).render(
  <StrictMode>
    <QueryClientProvider client={consultas}>
      <HashRouter>
        <App />
      </HashRouter>
    </QueryClientProvider>
  </StrictMode>,
);

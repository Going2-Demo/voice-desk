import { createRoot } from "react-dom/client";
import "../../compartilhado/tema/tokens.css";
import { itens } from "./itens";

const raiz = document.getElementById("raiz");
if (raiz) {
  createRoot(raiz).render(
    <ul aria-label="Menu do ícone">
      {itens.map((i) => (
        <li key={i.id}>{i.rotulo}</li>
      ))}
    </ul>,
  );
}

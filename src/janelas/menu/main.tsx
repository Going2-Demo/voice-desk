import { createRoot } from "react-dom/client";
import "../../compartilhado/tema/tokens.css";
import "../../compartilhado/tema/componentes.css";
import "../../compartilhado/tema/formulario.css";
import { CaixaRapida } from "./caixa-rapida";
import "./caixa-rapida.css";

const raiz = document.getElementById("raiz");
if (raiz) {
  createRoot(raiz).render(<CaixaRapida />);
}

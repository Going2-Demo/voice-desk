import { createRoot } from "react-dom/client";
import "../../compartilhado/tema/tokens.css";
import { Gravador } from "./gravador";

const raiz = document.getElementById("raiz");
if (raiz) createRoot(raiz).render(<Gravador />);

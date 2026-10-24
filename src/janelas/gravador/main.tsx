import { createRoot } from "react-dom/client";
import "../../compartilhado/tema/tokens.css";

const raiz = document.getElementById("raiz");
if (raiz) createRoot(raiz).render(<p role="status">Gravador (F-01)</p>);

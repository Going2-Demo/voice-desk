import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { useNavigate } from "react-router";

/** O menu do ícone pede à janela principal para abrir uma rota (histórico, configurações). */
export function NavegarPeloIcone() {
  const navegar = useNavigate();
  useEffect(() => {
    if (!("__TAURI_INTERNALS__" in window)) return;
    const fim = listen<string>("bandeja:navegar", (e) => navegar(e.payload));
    return () => {
      fim.then((desligar) => desligar());
    };
  }, [navegar]);
  return null;
}

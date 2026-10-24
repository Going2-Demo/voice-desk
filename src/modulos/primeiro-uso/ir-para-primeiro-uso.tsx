import { useEffect, useRef } from "react";
import { useLocation, useNavigate } from "react-router";
import { type ApiPrimeiroUso, api as apiReal } from "./api";

/** Na primeira abertura em cada máquina leva à tela de primeiro uso, uma vez por execução. */
export function IrParaPrimeiroUso({ api = apiReal }: { api?: ApiPrimeiroUso }) {
  const navegar = useNavigate();
  const { pathname } = useLocation();
  const jaVerificou = useRef(false);

  useEffect(() => {
    if (jaVerificou.current) return;
    jaVerificou.current = true;
    if (pathname === "/primeiro-uso") return;
    if (!("__TAURI_INTERNALS__" in window) && import.meta.env.VITE_IPC !== "simulado") return;
    api
      .estado()
      .then((e) => {
        if (!e.concluido) navegar("/primeiro-uso", { replace: true });
      })
      .catch(() => {
        // Sem ler o estado, não atrapalha: o menu continua levando ao primeiro uso.
      });
  }, [api, navegar, pathname]);
  return null;
}

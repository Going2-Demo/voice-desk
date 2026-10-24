import { Suspense } from "react";
import { Navigate, Route, Routes } from "react-router";
import { NavegarPeloIcone } from "../modulos/bandeja/navegar-pelo-icone";
import { IrParaPrimeiroUso } from "../modulos/primeiro-uso/ir-para-primeiro-uso";
import { Lateral } from "./lateral";
import { rotaInicial, rotas } from "./rotas";

export function App() {
  return (
    <div className="app">
      <NavegarPeloIcone />
      <IrParaPrimeiroUso />
      <Lateral />
      <main className="conteudo">
        <Suspense fallback={<p>Carregando…</p>}>
          <Routes>
            {rotas.map(({ caminho, Pagina }) => (
              <Route key={caminho} path={caminho} element={<Pagina />} />
            ))}
            <Route path="*" element={<Navigate to={rotaInicial} replace />} />
          </Routes>
        </Suspense>
      </main>
    </div>
  );
}

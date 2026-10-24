import { NavLink } from "react-router";
import { rotas } from "./rotas";

const grupos = ["Ditado", "Texto", "Máquinas", "App"] as const;

export function Lateral() {
  return (
    <nav className="lateral" aria-label="Principal">
      {grupos.map((grupo) => (
        <div key={grupo}>
          <h2>{grupo}</h2>
          {rotas
            .filter((r) => r.grupo === grupo && r.naLateral)
            .map((r) => (
              <NavLink key={r.caminho} to={r.caminho}>
                {r.rotulo}
              </NavLink>
            ))}
        </div>
      ))}
    </nav>
  );
}

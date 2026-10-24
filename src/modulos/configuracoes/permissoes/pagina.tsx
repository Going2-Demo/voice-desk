import { ListaPermissoes } from "./permissoes";

export default function Pagina() {
  return (
    <section>
      <h1>Configurações · Permissões</h1>
      <div className="cartao">
        <h2>Permissões do sistema</h2>
        <p className="dica">
          O Voice Desk pede só o necessário. Se negou alguma, conceda aqui e o estado se atualiza.
        </p>
        <ListaPermissoes />
      </div>
    </section>
  );
}

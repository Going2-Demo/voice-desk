import { PaginaProvisoria } from "../../../compartilhado/ui/pagina-provisoria";
import { Secao as IniciarComOSistema } from "./secao-iniciar-com-o-sistema";

export default function Pagina() {
  return (
    <>
      <PaginaProvisoria titulo="Configurações · Geral" area="preferencias-gerais" />
      <IniciarComOSistema />
    </>
  );
}

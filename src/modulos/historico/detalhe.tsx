import { useParams } from "react-router";
import { PaginaProvisoria } from "../../compartilhado/ui/pagina-provisoria";

export default function Detalhe() {
  const { id } = useParams();
  return <PaginaProvisoria titulo={`Histórico · item ${id}`} area="historico" />;
}

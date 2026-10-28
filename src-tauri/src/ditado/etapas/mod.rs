//! Uma etapa por arquivo, todas "passa adiante" no F-00.

pub mod ao_iniciar;
pub mod ao_parar;
pub mod captura;
pub mod colar;
pub mod contexto;
pub mod descartar_sem_fala;
pub mod gatilho;
pub mod limpar;
pub mod refinar;
pub mod registrar;
pub mod substituir;
pub mod transcrever;

use super::colagem::ResultadoColagem;
use crate::audio::captura::Gravacao;
use crate::transcricao::{ConfigTranscricao, FabricaCompartilhada, ProvedorTranscricao};

/// Dados que atravessam as etapas. Cada história acrescenta os campos de que precisa.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Contexto {
    pub texto_transcrito: String,
    pub texto_final: String,
    /// Programa que estava em foco quando o ditado começou (RF-01, usado por modos e histórico).
    pub programa_em_foco: Option<String>,
    /// Modo pede Enter depois de colar (RF-03.3). Preenchido quando o modo ativo é resolvido.
    pub enviar_enter: bool,
    /// Resultado da colagem (RF-03.5); `None` se a etapa `colar` não rodou ou não havia texto.
    pub colagem: Option<ResultadoColagem>,
    /// Áudio capturado (some quando não houve captura).
    pub gravacao: Option<Gravacao>,
    /// Modo ativo e a configuração de transcrição dele (F-02).
    pub modo_id: Option<String>,
    pub config_transcricao: ConfigTranscricao,
    /// Dica de vocabulário enviada ao provedor (F-07); `None` com o vocabulário vazio.
    pub dica_vocabulario: Option<String>,
    /// Substituições ativas aplicadas pela etapa `substituir` (F-08).
    pub substituicoes: Vec<crate::substituicoes::Regra>,
    pub fabrica_transcritor: Option<FabricaCompartilhada>,
    pub transcricao_provedor: Option<ProvedorTranscricao>,
    pub transcricao_modelo: Option<String>,
    /// Nomes das etapas já executadas, na ordem (útil em teste e diagnóstico; sem texto ditado).
    pub passou_por: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fluxo {
    Seguir,
    /// Encerra o ditado sem erro (ex.: descartado por falta de fala).
    Parar,
}

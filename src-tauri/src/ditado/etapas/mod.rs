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

/// Dados que atravessam as etapas. Cada história acrescenta os campos de que precisa.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Contexto {
    pub texto_transcrito: String,
    pub texto_final: String,
    /// Programa que estava em foco quando o ditado começou (RF-01, usado por modos e histórico).
    pub programa_em_foco: Option<String>,
    /// Nomes das etapas já executadas, na ordem (útil em teste e diagnóstico; sem texto ditado).
    pub passou_por: Vec<&'static str>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fluxo {
    Seguir,
    /// Encerra o ditado sem erro (ex.: descartado por falta de fala).
    Parar,
}

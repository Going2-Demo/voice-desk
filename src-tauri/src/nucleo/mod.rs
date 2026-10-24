//! Base compartilhada: erro, banco e migrador, relógio lógico, caminhos, log.

pub mod banco;
pub mod caminhos;
pub mod erro;
pub mod log;
pub mod relogio;
pub mod texto;

pub use erro::ErroApp;
pub use texto::normalizar;

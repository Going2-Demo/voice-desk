//! Relógio lógico híbrido: texto ordenável para `atualizado_em` (arquitetura, seção 4).
//! Formato: `<ms desde 1970, 13 dígitos>-<contador, 6 dígitos>-<id da máquina>`.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Relogio {
    maquina: String,
    ultimo: Mutex<(u64, u32)>,
}

impl Relogio {
    pub fn novo(maquina: &str) -> Self {
        Self {
            maquina: maquina.into(),
            ultimo: Mutex::new((0, 0)),
        }
    }

    fn agora_ms() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Próximo carimbo: sempre maior que o anterior, mesmo se o relógio do sistema voltar.
    pub fn proximo(&self) -> String {
        self.proximo_em(Self::agora_ms())
    }

    pub(crate) fn proximo_em(&self, agora: u64) -> String {
        let mut ultimo = self.ultimo.lock().expect("relógio envenenado");
        *ultimo = if agora > ultimo.0 {
            (agora, 0)
        } else {
            (ultimo.0, ultimo.1 + 1)
        };
        format!("{:013}-{:06}-{}", ultimo.0, ultimo.1, self.maquina)
    }

    /// Absorve um carimbo recebido de outra máquina para que os próximos sejam maiores.
    pub fn observar(&self, carimbo: &str) {
        let mut partes = carimbo.splitn(3, '-');
        let (Some(ms), Some(cont)) = (partes.next(), partes.next()) else {
            return;
        };
        let (Ok(ms), Ok(cont)) = (ms.parse::<u64>(), cont.parse::<u32>()) else {
            return;
        };
        let mut ultimo = self.ultimo.lock().expect("relógio envenenado");
        if (ms, cont) > *ultimo {
            *ultimo = (ms, cont);
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn carimbos_crescem_mesmo_no_mesmo_milissegundo() {
        let r = Relogio::novo("m1");
        let (a, b) = (r.proximo_em(1000), r.proximo_em(1000));
        assert!(b > a);
    }

    #[test]
    fn carimbos_crescem_se_o_relogio_voltar() {
        let r = Relogio::novo("m1");
        let a = r.proximo_em(2000);
        let b = r.proximo_em(1000);
        assert!(b > a);
    }

    #[test]
    fn observar_carimbo_futuro_empurra_o_proximo() {
        let r = Relogio::novo("m1");
        r.observar("0000000009000-000003-m2");
        assert!(r.proximo_em(1000).as_str() > "0000000009000-000003-m2");
    }

    #[test]
    fn observar_lixo_nao_quebra() {
        Relogio::novo("m1").observar("lixo");
    }
}

//! Descartar ditado sem fala (F-26). Detector de energia por quadros de 20 ms sobre a gravação e
//! a preferência local `audio.descartar_sem_fala` (RF-26.1, padrão ligado).

use crate::audio::captura::Gravacao;
use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;

pub const CHAVE: &str = "audio.descartar_sem_fala";

const QUADRO_MS: u32 = 20;
/// Quadro com RMS acima disto (dBFS) conta como som; fala baixa fica bem acima, ruído de fundo abaixo.
const LIMIAR_DBFS: f32 = -50.0;
/// Tempo mínimo de quadros com som para considerar que houve fala.
const MINIMO_COM_SOM_MS: u32 = 200;

/// A gravação tem fala? Silêncio, ruído muito baixo e estalos curtos não contam.
pub fn tem_fala(gravacao: &Gravacao) -> bool {
    let tamanho = (gravacao.taxa_hz * QUADRO_MS / 1000).max(1) as usize;
    let minimo = (MINIMO_COM_SOM_MS / QUADRO_MS) as usize;
    let com_som = gravacao
        .amostras
        .chunks(tamanho)
        .filter(|q| rms_dbfs(q) > LIMIAR_DBFS)
        .count();
    com_som >= minimo
}

fn rms_dbfs(quadro: &[i16]) -> f32 {
    if quadro.is_empty() {
        return f32::NEG_INFINITY;
    }
    let soma: f64 = quadro
        .iter()
        .map(|a| {
            let v = f64::from(*a) / f64::from(i16::MAX);
            v * v
        })
        .sum();
    let rms = (soma / quadro.len() as f64).sqrt();
    if rms <= 0.0 {
        f32::NEG_INFINITY
    } else {
        (20.0 * rms.log10()) as f32
    }
}

/// Preferência local; sem registro, o padrão é ligado.
pub fn ler(conexao: &Connection) -> Result<bool, ErroApp> {
    let valor: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = ?1",
            [CHAVE],
            |l| l.get(0),
        )
        .optional()?;
    Ok(valor.as_deref().map(|v| v != "false").unwrap_or(true))
}

pub fn gravar(conexao: &Connection, ligado: bool) -> Result<bool, ErroApp> {
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        rusqlite::params![CHAVE, if ligado { "true" } else { "false" }],
    )?;
    Ok(ligado)
}

#[tauri::command]
#[specta::specta]
pub fn descartar_sem_fala_obter(banco: tauri::State<Mutex<Connection>>) -> Result<bool, ErroApp> {
    ler(&*banco.lock().map_err(ErroApp::interno)?)
}

#[tauri::command]
#[specta::specta]
pub fn descartar_sem_fala_definir(
    banco: tauri::State<Mutex<Connection>>,
    ligado: bool,
) -> Result<bool, ErroApp> {
    gravar(&*banco.lock().map_err(ErroApp::interno)?, ligado)
}

#[cfg(test)]
mod testes {
    use super::*;

    const TAXA: u32 = 16_000;

    fn gravacao(amostras: Vec<i16>) -> Gravacao {
        Gravacao {
            amostras,
            taxa_hz: TAXA,
        }
    }

    /// Onda quadrada de amplitude `amp` por `ms` milissegundos.
    fn tom(amp: i16, ms: u32) -> Vec<i16> {
        (0..TAXA * ms / 1000)
            .map(|i| if i % 2 == 0 { amp } else { -amp })
            .collect()
    }

    #[test]
    fn cinco_segundos_de_silencio_digital_nao_tem_fala() {
        assert!(!tem_fala(&gravacao(vec![0; (TAXA * 5) as usize])));
    }

    #[test]
    fn ruido_de_fundo_muito_baixo_nao_tem_fala() {
        // amplitude 20/32767 ≈ -64 dBFS
        assert!(!tem_fala(&gravacao(tom(20, 5000))));
    }

    #[test]
    fn gravacao_vazia_nao_tem_fala() {
        assert!(!tem_fala(&gravacao(vec![])));
    }

    #[test]
    fn fala_baixa_mas_audivel_tem_fala() {
        // amplitude 300/32767 ≈ -41 dBFS, por 1 s no meio de silêncio
        let mut a = vec![0; TAXA as usize];
        a.extend(tom(300, 1000));
        a.extend(vec![0; TAXA as usize]);
        assert!(tem_fala(&gravacao(a)));
    }

    #[test]
    fn fala_normal_tem_fala() {
        assert!(tem_fala(&gravacao(tom(4000, 800))));
    }

    #[test]
    fn estalo_curto_nao_tem_fala() {
        let mut a = vec![0; (TAXA * 3) as usize];
        a.extend(tom(10_000, 40));
        assert!(!tem_fala(&gravacao(a)));
    }

    fn banco() -> Connection {
        let c = Connection::open_in_memory().unwrap();
        c.execute(
            "CREATE TABLE preferencia_local (chave TEXT PRIMARY KEY, valor TEXT NOT NULL)",
            [],
        )
        .unwrap();
        c
    }

    #[test]
    fn preferencia_vem_ligada_e_guarda_a_escolha() {
        let c = banco();
        assert!(ler(&c).unwrap());
        assert!(!gravar(&c, false).unwrap());
        assert!(!ler(&c).unwrap());
        assert!(gravar(&c, true).unwrap());
        assert!(ler(&c).unwrap());
    }
}

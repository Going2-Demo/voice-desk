//! Sons de início e fim do ditado (F-24). Os três sons são sintetizados aqui (tons próprios do
//! Voice Desk, nada do VoiceInk). A preferência é local (`preferencia_local`, chave `audio.sons`).
//! O som de início toca até o fim antes de a captura abrir, então não entra na gravação (RF-24.2).

use crate::nucleo::ErroApp;
use rusqlite::{Connection, OptionalExtension};
use serde::{Deserialize, Serialize};
use specta::Type;
use std::f32::consts::TAU;
use std::sync::Mutex;

pub const CHAVE: &str = "audio.sons";
const TAXA: u32 = 44_100;
const VOLUME: f32 = 0.35;

/// Sons disponíveis: `(id, nome na tela)`.
pub const SONS: [(&str, &str); 3] = [
    ("gota", "Gota"),
    ("pulso", "Pulso"),
    ("clique_suave", "Clique suave"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Momento {
    Inicio,
    Fim,
}

/// Preferência de sons: padrão ligado, som "Gota".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct PreferenciaSons {
    pub ligado: bool,
    pub som: String,
    /// `(id, nome)` dos sons que o usuário pode escolher.
    #[serde(default)]
    pub opcoes: Vec<OpcaoSom>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Type)]
pub struct OpcaoSom {
    pub id: String,
    pub nome: String,
}

#[derive(Serialize, Deserialize)]
struct Guardada {
    ligado: bool,
    som: String,
}

fn opcoes() -> Vec<OpcaoSom> {
    SONS.iter()
        .map(|(id, nome)| OpcaoSom {
            id: (*id).into(),
            nome: (*nome).into(),
        })
        .collect()
}

fn som_valido(id: &str) -> bool {
    SONS.iter().any(|(s, _)| *s == id)
}

/// Valor guardado; sem registro ou ilegível, o padrão (ligado, Gota).
pub fn ler(conexao: &Connection) -> Result<PreferenciaSons, ErroApp> {
    let valor: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = ?1",
            [CHAVE],
            |l| l.get(0),
        )
        .optional()?;
    let guardada = valor
        .and_then(|v| serde_json::from_str::<Guardada>(&v).ok())
        .filter(|g| som_valido(&g.som))
        .unwrap_or(Guardada {
            ligado: true,
            som: SONS[0].0.into(),
        });
    Ok(PreferenciaSons {
        ligado: guardada.ligado,
        som: guardada.som,
        opcoes: opcoes(),
    })
}

pub fn gravar(conexao: &Connection, ligado: bool, som: &str) -> Result<PreferenciaSons, ErroApp> {
    if !som_valido(som) {
        return Err(
            ErroApp::novo("som_invalido", "Este som não existe. Escolha um da lista.")
                .com_campo("som"),
        );
    }
    let json = serde_json::to_string(&Guardada {
        ligado,
        som: som.into(),
    })
    .map_err(ErroApp::interno)?;
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES (?1, ?2)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        rusqlite::params![CHAVE, json],
    )?;
    ler(conexao)
}

/// Trechos de cada som: `(frequência inicial, frequência final, duração em ms)`; a lista é
/// tocada em sequência, cada trecho com ataque curto e queda exponencial. O fim é a versão
/// "de volta" do início, para os dois serem distinguíveis sem olhar.
fn trechos(som: &str, momento: Momento) -> Vec<(f32, f32, u32)> {
    let inicio = momento == Momento::Inicio;
    match (som, inicio) {
        ("pulso", true) => vec![(523.0, 523.0, 70), (0.0, 0.0, 40), (659.0, 659.0, 90)],
        ("pulso", false) => vec![(659.0, 659.0, 70), (0.0, 0.0, 40), (523.0, 523.0, 90)],
        ("clique_suave", true) => vec![(1500.0, 1100.0, 45)],
        ("clique_suave", false) => vec![(1100.0, 800.0, 55)],
        (_, true) => vec![(660.0, 990.0, 170)],
        (_, false) => vec![(990.0, 620.0, 210)],
    }
}

/// Amostras mono (44,1 kHz) do som. Id desconhecido cai no primeiro som.
pub fn amostras(som: &str, momento: Momento) -> Vec<f32> {
    let mut saida = Vec::new();
    for (f0, f1, ms) in trechos(som, momento) {
        let n = (TAXA as usize * ms as usize) / 1000;
        if f0 == 0.0 {
            saida.extend(std::iter::repeat_n(0.0, n));
            continue;
        }
        let mut fase = 0.0_f32;
        for i in 0..n {
            let t = i as f32 / n as f32;
            fase += TAU * (f0 + (f1 - f0) * t) / TAXA as f32;
            let ataque = (i as f32 / (TAXA as f32 * 0.004)).min(1.0);
            let queda = (-3.0 * t).exp() * ((1.0 - t) / 0.1).min(1.0);
            saida.push(fase.sin() * VOLUME * ataque * queda);
        }
    }
    saida
}

/// Toca amostras e só volta quando o som terminou.
pub trait Reprodutor: Send + Sync {
    fn tocar(&self, amostras: &[f32], taxa: u32) -> Result<(), String>;
}

/// Reprodução pelo dispositivo de saída padrão (rodio).
pub struct Rodio;

impl Reprodutor for Rodio {
    fn tocar(&self, amostras: &[f32], taxa: u32) -> Result<(), String> {
        use std::num::NonZero;
        let mut saida = rodio::DeviceSinkBuilder::open_default_sink().map_err(|e| e.to_string())?;
        saida.log_on_drop(false);
        let player = rodio::Player::connect_new(saida.mixer());
        let (canais, taxa) = (
            NonZero::new(1).expect("1"),
            NonZero::new(taxa).ok_or("taxa zero")?,
        );
        player.append(rodio::buffer::SamplesBuffer::new(
            canais,
            taxa,
            amostras.to_vec(),
        ));
        player.sleep_until_end();
        Ok(())
    }
}

/// Toca o som do momento se a preferência está ligada. Falha de reprodução só vai ao log: o
/// ditado nunca depende do som. Devolve se tocou.
pub fn tocar_se_ligado(
    banco: &Mutex<Connection>,
    reprodutor: &dyn Reprodutor,
    momento: Momento,
) -> bool {
    let pref = match banco.lock().ok().and_then(|c| ler(&c).ok()) {
        Some(p) => p,
        None => return false,
    };
    if !pref.ligado {
        return false;
    }
    tocar(reprodutor, &pref.som, momento)
}

pub fn tocar(reprodutor: &dyn Reprodutor, som: &str, momento: Momento) -> bool {
    match reprodutor.tocar(&amostras(som, momento), TAXA) {
        Ok(()) => true,
        Err(e) => {
            tracing::warn!(evento = "som_falhou", detalhe = %e);
            false
        }
    }
}

#[tauri::command]
#[specta::specta]
pub fn sons_obter(banco: tauri::State<Mutex<Connection>>) -> Result<PreferenciaSons, ErroApp> {
    ler(&*banco.lock().map_err(ErroApp::interno)?)
}

#[tauri::command]
#[specta::specta]
pub fn sons_definir(
    banco: tauri::State<Mutex<Connection>>,
    ligado: bool,
    som: String,
) -> Result<PreferenciaSons, ErroApp> {
    gravar(&*banco.lock().map_err(ErroApp::interno)?, ligado, &som)
}

/// Prévia (RF-24.1): toca o som de início escolhido, mesmo que a escolha ainda não tenha sido
/// salva. Com os sons desligados, o botão fica desabilitado na tela e aqui nada toca.
#[tauri::command]
#[specta::specta]
pub async fn sons_previa(
    banco: tauri::State<'_, Mutex<Connection>>,
    som: String,
) -> Result<(), ErroApp> {
    if !som_valido(&som) {
        return Err(
            ErroApp::novo("som_invalido", "Este som não existe. Escolha um da lista.")
                .com_campo("som"),
        );
    }
    let ligado = ler(&*banco.lock().map_err(ErroApp::interno)?)?.ligado;
    if ligado {
        tauri::async_runtime::spawn_blocking(move || tocar(&Rodio, &som, Momento::Inicio));
    }
    Ok(())
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::nucleo::banco;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[derive(Default)]
    struct Falso {
        tocados: AtomicUsize,
        falha: bool,
    }
    impl Reprodutor for Falso {
        fn tocar(&self, amostras: &[f32], _: u32) -> Result<(), String> {
            assert!(!amostras.is_empty());
            if self.falha {
                return Err("sem saída de áudio".into());
            }
            self.tocados.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    fn mem() -> Mutex<Connection> {
        let mut c = Connection::open_in_memory().unwrap();
        banco::migrar(&mut c, banco::MIGRACOES).unwrap();
        Mutex::new(c)
    }

    #[test]
    fn padrao_ligado_com_gota_e_tres_opcoes() {
        let b = mem();
        let p = ler(&b.lock().unwrap()).unwrap();
        assert!(p.ligado);
        assert_eq!(p.som, "gota");
        assert_eq!(p.opcoes.len(), 3);
    }

    #[test]
    fn grava_e_le_a_escolha() {
        let b = mem();
        gravar(&b.lock().unwrap(), false, "pulso").unwrap();
        let p = ler(&b.lock().unwrap()).unwrap();
        assert!(!p.ligado);
        assert_eq!(p.som, "pulso");
    }

    #[test]
    fn som_desconhecido_e_recusado_sem_gravar() {
        let b = mem();
        let e = gravar(&b.lock().unwrap(), true, "vuvuzela").unwrap_err();
        assert_eq!(e.codigo, "som_invalido");
        assert_eq!(e.campo.as_deref(), Some("som"));
        assert_eq!(ler(&b.lock().unwrap()).unwrap().som, "gota");
    }

    #[test]
    fn registro_corrompido_volta_ao_padrao() {
        let b = mem();
        b.lock()
            .unwrap()
            .execute(
                "INSERT INTO preferencia_local (chave, valor) VALUES (?1, '{\"som\":\"vuvuzela\"}')",
                [CHAVE],
            )
            .unwrap();
        let p = ler(&b.lock().unwrap()).unwrap();
        assert!(p.ligado);
        assert_eq!(p.som, "gota");
    }

    #[test]
    fn ligado_toca_inicio_e_fim() {
        let b = mem();
        let r = Falso::default();
        assert!(tocar_se_ligado(&b, &r, Momento::Inicio));
        assert!(tocar_se_ligado(&b, &r, Momento::Fim));
        assert_eq!(r.tocados.load(Ordering::SeqCst), 2);
    }

    #[test]
    fn desligado_nao_toca_nada() {
        let b = mem();
        gravar(&b.lock().unwrap(), false, "gota").unwrap();
        let r = Falso::default();
        assert!(!tocar_se_ligado(&b, &r, Momento::Inicio));
        assert!(!tocar_se_ligado(&b, &r, Momento::Fim));
        assert_eq!(r.tocados.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn falha_de_reproducao_nao_derruba_o_ditado() {
        let b = mem();
        let r = Falso {
            falha: true,
            ..Falso::default()
        };
        assert!(!tocar_se_ligado(&b, &r, Momento::Inicio));
    }

    #[test]
    fn cada_som_tem_amostras_audiveis_e_inicio_difere_do_fim() {
        for (id, _) in SONS {
            let ini = amostras(id, Momento::Inicio);
            let fim = amostras(id, Momento::Fim);
            assert!(ini.len() > 1000 && fim.len() > 1000, "{id}");
            assert!(ini.iter().any(|a| a.abs() > 0.05), "{id} inaudível");
            assert!(ini.iter().all(|a| a.abs() <= 1.0));
            assert_ne!(ini, fim, "{id}");
            assert!(ini.len() < TAXA as usize / 2, "{id} longo demais");
        }
    }
}

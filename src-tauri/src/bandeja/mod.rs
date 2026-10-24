//! Ícone na barra de menus (Mac) ou na bandeja (Windows), menu rápido e "iniciar com o sistema"
//! (F-05). O app vive aqui, sem janela aberta; o estado do ícone vem de quem muda o estado do
//! ditado, por `definir_estado`.

pub mod estado;
pub mod menu;
pub mod preferencia;

use crate::nucleo::{ErroApp, relogio::Relogio};
use estado::EstadoIcone;
use menu::{ItemModelo, ModoMenu};
use preferencia::InicioAutomatico;
use rusqlite::{Connection, OptionalExtension};
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{CheckMenuItem, IsMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use tauri::tray::{TrayIcon, TrayIconBuilder};
use tauri::{AppHandle, Emitter, Manager, Wry};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};

/// Argumento passado quando o sistema abre o app no login: sobe só o ícone, sem janela.
pub const ARGUMENTO_OCULTO: &str = "--oculto";
const ID_ICONE: &str = "bandeja";
/// Evento para o ditado (F-01): o usuário pediu iniciar ou parar pelo menu.
pub const EVENTO_ALTERNAR_DITADO: &str = "bandeja:alternar_ditado";
/// Evento para a janela principal: navegar para a rota recebida (ex.: `/historico`).
pub const EVENTO_NAVEGAR: &str = "bandeja:navegar";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Acao {
    AlternarDitado,
    EscolherModo(String),
    AbrirHistorico,
    AbrirConfiguracoes,
    Sair,
}

pub fn acao_do_item(id: &str) -> Option<Acao> {
    match id {
        menu::ID_DITADO => Some(Acao::AlternarDitado),
        menu::ID_HISTORICO => Some(Acao::AbrirHistorico),
        menu::ID_CONFIG => Some(Acao::AbrirConfiguracoes),
        menu::ID_SAIR => Some(Acao::Sair),
        outro => menu::id_do_modo(outro).map(|m| Acao::EscolherModo(m.to_string())),
    }
}

pub struct Bandeja {
    icone: TrayIcon<Wry>,
    estado: Mutex<EstadoIcone>,
}

struct SistemaAutostart(AppHandle);

impl InicioAutomatico for SistemaAutostart {
    fn ligado(&self) -> Result<bool, ErroApp> {
        self.0.autolaunch().is_enabled().map_err(ErroApp::interno)
    }
    fn definir(&self, ligado: bool) -> Result<(), ErroApp> {
        let a = self.0.autolaunch();
        if ligado { a.enable() } else { a.disable() }.map_err(ErroApp::interno)
    }
}

type Banco = Mutex<Connection>;

fn banco(app: &AppHandle) -> &Banco {
    app.state::<Banco>().inner()
}

/// Identificador estável desta máquina (carimbos de sincronização).
pub(crate) fn maquina_id(conexao: &Connection) -> Result<String, ErroApp> {
    let atual: Option<String> = conexao
        .query_row(
            "SELECT valor FROM preferencia_local WHERE chave = 'maquina_id'",
            [],
            |l| l.get(0),
        )
        .optional()?;
    if let Some(json) = atual.and_then(|j| serde_json::from_str::<String>(&j).ok()) {
        return Ok(json);
    }
    let novo = uuid::Uuid::now_v7().to_string();
    conexao.execute(
        "INSERT INTO preferencia_local (chave, valor) VALUES ('maquina_id', ?1)
         ON CONFLICT (chave) DO UPDATE SET valor = excluded.valor",
        [serde_json::to_string(&novo).map_err(ErroApp::interno)?],
    )?;
    Ok(novo)
}

fn montar_menu(app: &AppHandle, estado: EstadoIcone) -> Result<Menu<Wry>, ErroApp> {
    let modos: Vec<ModoMenu> = {
        let conexao = banco(app).lock().map_err(ErroApp::interno)?;
        menu::listar_modos(&conexao)?
    };
    let erro = ErroApp::interno;
    let raiz = Menu::new(app).map_err(erro)?;
    for item in menu::modelo(estado, modos) {
        match item {
            ItemModelo::Acao { id, rotulo } => {
                let i = MenuItem::with_id(app, id, rotulo, true, None::<&str>).map_err(erro)?;
                raiz.append(&i).map_err(erro)?;
            }
            ItemModelo::Separador => {
                raiz.append(&PredefinedMenuItem::separator(app).map_err(erro)?)
                    .map_err(erro)?;
            }
            ItemModelo::Modos(modos) => {
                let checks: Vec<CheckMenuItem<Wry>> = modos
                    .iter()
                    .map(|m| {
                        CheckMenuItem::with_id(
                            app,
                            format!("{}{}", menu::PREFIXO_MODO, m.id),
                            &m.rotulo,
                            true,
                            m.padrao,
                            None::<&str>,
                        )
                    })
                    .collect::<Result<_, _>>()
                    .map_err(erro)?;
                let refs: Vec<&dyn IsMenuItem<Wry>> =
                    checks.iter().map(|c| c as &dyn IsMenuItem<Wry>).collect();
                let sub = Submenu::with_items(app, "Modo padrão", true, &refs).map_err(erro)?;
                raiz.append(&sub).map_err(erro)?;
            }
        }
    }
    Ok(raiz)
}

fn aplicar_estado(app: &AppHandle, estado: EstadoIcone) -> Result<(), ErroApp> {
    let bandeja = app.state::<Bandeja>();
    let imagem = Image::from_bytes(estado.png()).map_err(ErroApp::interno)?;
    bandeja
        .icone
        .set_icon(Some(imagem))
        .map_err(ErroApp::interno)?;
    // Só o ícone parado é "template" no Mac (segue o tema da barra); os demais trazem cor.
    let _ = bandeja
        .icone
        .set_icon_as_template(estado == EstadoIcone::Parado);
    bandeja
        .icone
        .set_tooltip(Some(estado.dica()))
        .map_err(ErroApp::interno)?;
    let novo_menu = montar_menu(app, estado)?;
    bandeja
        .icone
        .set_menu(Some(novo_menu))
        .map_err(ErroApp::interno)?;
    *bandeja.estado.lock().map_err(ErroApp::interno)? = estado;
    Ok(())
}

/// Muda a aparência do ícone (parado, gravando, processando, erro) e o item de ditado do menu.
pub fn definir_estado(app: &AppHandle, estado: EstadoIcone) -> Result<(), ErroApp> {
    aplicar_estado(app, estado)
}

/// Mostra a janela principal (recriada se foi fechada) e, se houver rota, navega até ela.
pub fn abrir_principal(app: &AppHandle, rota: Option<&str>) {
    let janela = match app.get_webview_window("principal") {
        Some(j) => j,
        None => {
            let Some(config) = app.config().app.windows.first().cloned() else {
                return;
            };
            match tauri::WebviewWindowBuilder::from_config(app, &config)
                .and_then(|b| b.visible(true).build())
            {
                Ok(j) => j,
                Err(e) => {
                    tracing::error!(evento = "janela_principal_falhou", detalhe = %e);
                    return;
                }
            }
        }
    };
    let _ = janela.show();
    let _ = janela.unminimize();
    let _ = janela.set_focus();
    if let Some(rota) = rota {
        let _ = app.emit_to("principal", EVENTO_NAVEGAR, rota);
    }
}

fn confirmar_e_sair(app: &AppHandle) {
    let app = app.clone();
    app.dialog()
        .message("Ao sair, o ícone some e o atalho global deixa de funcionar até você abrir o Voice Desk de novo.")
        .title("Sair do Voice Desk?")
        .kind(MessageDialogKind::Warning)
        .buttons(MessageDialogButtons::OkCancelCustom("Sair".into(), "Cancelar".into()))
        .show({
            let app = app.clone();
            move |confirmou| {
                if confirmou {
                    tracing::info!(evento = "app_encerrado_pelo_menu");
                    app.exit(0);
                }
            }
        });
}

fn executar(app: &AppHandle, acao: Acao) {
    match acao {
        Acao::AlternarDitado => {
            let _ = app.emit(EVENTO_ALTERNAR_DITADO, ());
        }
        Acao::EscolherModo(id) => {
            let resultado = (|| {
                let conexao = banco(app).lock().map_err(ErroApp::interno)?;
                let maquina = maquina_id(&conexao)?;
                menu::escolher_modo(&conexao, &app.state::<Relogio>(), &maquina, &id)
            })();
            if let Err(e) = resultado {
                tracing::warn!(evento = "modo_padrao_falhou", codigo = %e.codigo);
            }
            let estado = *app
                .state::<Bandeja>()
                .estado
                .lock()
                .expect("estado envenenado");
            let _ = aplicar_estado(app, estado);
        }
        Acao::AbrirHistorico => abrir_principal(app, Some("/historico")),
        Acao::AbrirConfiguracoes => abrir_principal(app, Some("/config/geral")),
        Acao::Sair => confirmar_e_sair(app),
    }
}

/// Cria o ícone e sincroniza "iniciar com o sistema". Chamada no `setup`, com o banco já aberto.
pub fn iniciar(app: &AppHandle) -> Result<(), ErroApp> {
    let maquina = maquina_id(&*banco(app).lock().map_err(ErroApp::interno)?)?;
    app.manage(Relogio::novo(&maquina));
    let menu_inicial = montar_menu(app, EstadoIcone::Parado)?;
    let icone = TrayIconBuilder::with_id(ID_ICONE)
        .icon(Image::from_bytes(EstadoIcone::Parado.png()).map_err(ErroApp::interno)?)
        .icon_as_template(true)
        .tooltip(EstadoIcone::Parado.dica())
        .menu(&menu_inicial)
        .on_menu_event(|app, evento| {
            if let Some(acao) = acao_do_item(evento.id().as_ref()) {
                executar(app, acao);
            }
        })
        .build(app)
        .map_err(ErroApp::interno)?;
    app.manage(Bandeja {
        icone,
        estado: Mutex::new(EstadoIcone::Parado),
    });
    let conexao = banco(app).lock().map_err(ErroApp::interno)?;
    if let Err(e) = preferencia::reconciliar(&conexao, &SistemaAutostart(app.clone())) {
        tracing::warn!(evento = "inicio_automatico_falhou", codigo = %e.codigo);
    }
    Ok(())
}

#[tauri::command]
#[specta::specta]
pub fn bandeja_estado(bandeja: tauri::State<Bandeja>) -> Result<EstadoIcone, ErroApp> {
    Ok(*bandeja.estado.lock().map_err(ErroApp::interno)?)
}

#[tauri::command]
#[specta::specta]
pub fn bandeja_iniciar_com_o_sistema(banco: tauri::State<Banco>) -> Result<bool, ErroApp> {
    preferencia::ler(&*banco.lock().map_err(ErroApp::interno)?)
}

#[tauri::command]
#[specta::specta]
pub fn bandeja_definir_iniciar_com_o_sistema(
    app: AppHandle,
    banco: tauri::State<Banco>,
    ligado: bool,
) -> Result<bool, ErroApp> {
    let conexao = banco.lock().map_err(ErroApp::interno)?;
    preferencia::alterar(&conexao, &SistemaAutostart(app), ligado)
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn cada_item_do_menu_leva_a_sua_acao() {
        assert_eq!(acao_do_item("ditado"), Some(Acao::AlternarDitado));
        assert_eq!(acao_do_item("historico"), Some(Acao::AbrirHistorico));
        assert_eq!(acao_do_item("config"), Some(Acao::AbrirConfiguracoes));
        assert_eq!(acao_do_item("sair"), Some(Acao::Sair));
        assert_eq!(acao_do_item("modo:x"), Some(Acao::EscolherModo("x".into())));
        assert_eq!(acao_do_item("desconhecido"), None);
    }

    #[test]
    fn maquina_id_e_estavel() {
        let mut c = Connection::open_in_memory().unwrap();
        crate::nucleo::banco::migrar(&mut c, crate::nucleo::banco::MIGRACOES).unwrap();
        let a = maquina_id(&c).unwrap();
        assert_eq!(a, maquina_id(&c).unwrap());
    }
}

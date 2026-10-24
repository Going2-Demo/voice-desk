//! Máquina de estados do ditado (RF-01.2 a RF-01.6). Pura: o tempo entra como milissegundos e a
//! saída é a ação que o orquestrador executa. Sem I/O, para ser testada sem sistema operacional.

pub const LIMITE_MS: u64 = 10 * 60 * 1000;
pub const AVISO_MS: u64 = LIMITE_MS - 30 * 1000;
pub const JANELA_ESC_MS: u64 = 1500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fase {
    Ocioso,
    Gravando,
    Processando,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acao {
    Nenhuma,
    /// Abrir o gravador e começar a capturar áudio.
    Iniciar,
    /// Parar a captura e seguir para o processamento (parada do usuário).
    Parar,
    /// Parar porque chegou aos 10 minutos (mesmo caminho da parada do usuário).
    PararPorLimite,
    /// Descartar a gravação: nada colado, nada enviado, nada no histórico.
    Cancelar,
    /// Atalho durante o processamento: o gravador indica "aguarde".
    Aguarde,
    /// Faltam 30 s para o limite.
    AvisarLimite,
}

#[derive(Debug, Clone, Copy)]
pub struct Maquina {
    fase: Fase,
    inicio_ms: u64,
    aviso_dado: bool,
    ultimo_esc_ms: Option<u64>,
}

impl Default for Maquina {
    fn default() -> Self {
        Self::nova()
    }
}

impl Maquina {
    pub const fn nova() -> Self {
        Self {
            fase: Fase::Ocioso,
            inicio_ms: 0,
            aviso_dado: false,
            ultimo_esc_ms: None,
        }
    }

    pub const fn fase(&self) -> Fase {
        self.fase
    }

    pub fn decorrido_ms(&self, agora_ms: u64) -> u64 {
        if self.fase == Fase::Gravando {
            agora_ms.saturating_sub(self.inicio_ms)
        } else {
            0
        }
    }

    /// O atalho global foi acionado.
    pub fn atalho(&mut self, agora_ms: u64) -> Acao {
        match self.fase {
            Fase::Ocioso => {
                *self = Self {
                    fase: Fase::Gravando,
                    inicio_ms: agora_ms,
                    ..Self::nova()
                };
                Acao::Iniciar
            }
            Fase::Gravando => {
                self.fase = Fase::Processando;
                Acao::Parar
            }
            Fase::Processando => Acao::Aguarde,
        }
    }

    /// Esc foi pressionado (a tecla não é consumida: o programa em foco a recebe normalmente).
    pub fn esc(&mut self, agora_ms: u64) -> Acao {
        if self.fase != Fase::Gravando {
            return Acao::Nenhuma;
        }
        match self.ultimo_esc_ms {
            Some(antes) if agora_ms.saturating_sub(antes) <= JANELA_ESC_MS => {
                *self = Self::nova();
                Acao::Cancelar
            }
            _ => {
                self.ultimo_esc_ms = Some(agora_ms);
                Acao::Nenhuma
            }
        }
    }

    /// Passagem do tempo (chamada a cada ~200 ms enquanto grava).
    pub fn tique(&mut self, agora_ms: u64) -> Acao {
        if self.fase != Fase::Gravando {
            return Acao::Nenhuma;
        }
        let decorrido = self.decorrido_ms(agora_ms);
        if decorrido >= LIMITE_MS {
            self.fase = Fase::Processando;
            Acao::PararPorLimite
        } else if decorrido >= AVISO_MS && !self.aviso_dado {
            self.aviso_dado = true;
            Acao::AvisarLimite
        } else {
            Acao::Nenhuma
        }
    }

    /// O processamento terminou (com sucesso ou não): o atalho volta a iniciar ditados.
    pub fn concluir(&mut self) {
        if self.fase == Fase::Processando {
            *self = Self::nova();
        }
    }

    /// A captura não pôde começar (ex.: sem permissão do microfone): nada foi gravado.
    pub fn falha_ao_iniciar(&mut self) {
        if self.fase == Fase::Gravando {
            *self = Self::nova();
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn gravando(t: u64) -> Maquina {
        let mut m = Maquina::nova();
        assert_eq!(m.atalho(t), Acao::Iniciar);
        m
    }

    #[test]
    fn atalho_inicia_e_depois_para() {
        let mut m = gravando(0);
        assert_eq!(m.fase(), Fase::Gravando);
        assert_eq!(m.atalho(5_000), Acao::Parar);
        assert_eq!(m.fase(), Fase::Processando);
    }

    #[test]
    fn durante_o_processamento_o_atalho_nao_inicia_outro() {
        let mut m = gravando(0);
        m.atalho(1_000);
        assert_eq!(m.atalho(2_000), Acao::Aguarde);
        assert_eq!(m.fase(), Fase::Processando);
        m.concluir();
        assert_eq!(m.fase(), Fase::Ocioso);
        assert_eq!(m.atalho(3_000), Acao::Iniciar);
    }

    #[test]
    fn esc_duas_vezes_em_ate_1500ms_cancela() {
        let mut m = gravando(0);
        assert_eq!(m.esc(1_000), Acao::Nenhuma);
        assert_eq!(m.esc(2_000), Acao::Cancelar);
        assert_eq!(m.fase(), Fase::Ocioso);
    }

    #[test]
    fn esc_uma_vez_so_nao_cancela() {
        let mut m = gravando(0);
        assert_eq!(m.esc(1_000), Acao::Nenhuma);
        assert_eq!(m.fase(), Fase::Gravando);
    }

    #[test]
    fn esc_com_mais_de_1500ms_entre_eles_conta_como_primeiro_de_novo() {
        let mut m = gravando(0);
        assert_eq!(m.esc(1_000), Acao::Nenhuma);
        assert_eq!(m.esc(2_501), Acao::Nenhuma);
        assert_eq!(m.esc(3_000), Acao::Cancelar);
    }

    #[test]
    fn esc_fora_da_gravacao_e_ignorado() {
        let mut m = Maquina::nova();
        assert_eq!(m.esc(0), Acao::Nenhuma);
        assert_eq!(m.esc(100), Acao::Nenhuma);
        let mut p = gravando(0);
        p.atalho(10);
        assert_eq!(p.esc(20), Acao::Nenhuma);
        assert_eq!(p.esc(30), Acao::Nenhuma);
        assert_eq!(p.fase(), Fase::Processando);
    }

    #[test]
    fn esc_de_uma_gravacao_anterior_nao_vaza_para_a_seguinte() {
        let mut m = gravando(0);
        m.esc(100);
        m.atalho(200);
        m.concluir();
        m.atalho(300);
        assert_eq!(m.esc(400), Acao::Nenhuma);
    }

    #[test]
    fn avisa_aos_9min30_uma_so_vez_e_para_aos_10min() {
        let mut m = gravando(1_000);
        assert_eq!(m.tique(1_000 + AVISO_MS - 1), Acao::Nenhuma);
        assert_eq!(m.tique(1_000 + AVISO_MS), Acao::AvisarLimite);
        assert_eq!(m.tique(1_000 + AVISO_MS + 200), Acao::Nenhuma);
        assert_eq!(m.tique(1_000 + LIMITE_MS), Acao::PararPorLimite);
        assert_eq!(m.fase(), Fase::Processando);
        assert_eq!(m.tique(1_000 + LIMITE_MS + 200), Acao::Nenhuma);
    }

    #[test]
    fn falha_ao_iniciar_volta_para_ocioso() {
        let mut m = gravando(0);
        m.falha_ao_iniciar();
        assert_eq!(m.fase(), Fase::Ocioso);
    }
}

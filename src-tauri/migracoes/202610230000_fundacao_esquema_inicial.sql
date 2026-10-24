-- Esquema inicial do Voice Desk (arquitetura, seção 4). Um arquivo SQLite por máquina.
-- Tabelas sincronizáveis: escrita só por configuracao::gravar().

CREATE TABLE modo (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  nome TEXT NOT NULL CHECK (length(nome) BETWEEN 1 AND 40),
  nome_normalizado TEXT NOT NULL,
  emoji TEXT NOT NULL DEFAULT '',
  idioma TEXT NOT NULL DEFAULT 'pt-BR' CHECK (idioma IN ('pt-BR', 'en', 'auto')),
  transcricao_provedor TEXT NOT NULL DEFAULT 'groq' CHECK (transcricao_provedor IN ('groq', 'whisper_local')),
  transcricao_modelo TEXT NOT NULL DEFAULT '',
  remover_preenchimento INTEGER NOT NULL DEFAULT 0,
  formatar INTEGER NOT NULL DEFAULT 0,
  refinar INTEGER NOT NULL DEFAULT 0,
  ia_provedor TEXT CHECK (ia_provedor IN ('groq', 'ollama')),
  ia_modelo TEXT NOT NULL DEFAULT '',
  prompt TEXT NOT NULL DEFAULT '' CHECK (length(prompt) <= 4000),
  contexto_selecao INTEGER NOT NULL DEFAULT 0,
  contexto_area_transferencia INTEGER NOT NULL DEFAULT 0,
  enviar_enter INTEGER NOT NULL DEFAULT 0,
  ordem INTEGER NOT NULL DEFAULT 0
);
CREATE UNIQUE INDEX modo_nome_unico ON modo (nome_normalizado) WHERE excluido_em IS NULL;

CREATE TABLE modo_programa (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  modo_id TEXT NOT NULL REFERENCES modo (id),
  plataforma TEXT NOT NULL CHECK (plataforma IN ('macos', 'windows')),
  identificador TEXT NOT NULL,
  nome_exibicao TEXT NOT NULL
);
CREATE UNIQUE INDEX modo_programa_unico ON modo_programa (plataforma, identificador) WHERE excluido_em IS NULL;

CREATE TABLE palavra_gatilho (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  modo_id TEXT NOT NULL REFERENCES modo (id),
  palavra TEXT NOT NULL,
  palavra_normalizada TEXT NOT NULL
);
CREATE UNIQUE INDEX palavra_gatilho_unica ON palavra_gatilho (palavra_normalizada) WHERE excluido_em IS NULL;

CREATE TABLE vocabulario (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  palavra TEXT NOT NULL,
  palavra_normalizada TEXT NOT NULL
);
CREATE UNIQUE INDEX vocabulario_unico ON vocabulario (palavra_normalizada) WHERE excluido_em IS NULL;

CREATE TABLE substituicao (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  original TEXT NOT NULL CHECK (length(trim(original)) > 0),
  original_normalizado TEXT NOT NULL,
  substituto TEXT NOT NULL,
  ativa INTEGER NOT NULL DEFAULT 1
);
CREATE UNIQUE INDEX substituicao_unica ON substituicao (original_normalizado) WHERE excluido_em IS NULL;

CREATE TABLE palavra_preenchimento (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  palavra TEXT NOT NULL,
  palavra_normalizada TEXT NOT NULL
);
CREATE UNIQUE INDEX palavra_preenchimento_unica ON palavra_preenchimento (palavra_normalizada) WHERE excluido_em IS NULL;

CREATE TABLE preferencia_sincronizada (
  id TEXT PRIMARY KEY NOT NULL,
  atualizado_em TEXT NOT NULL,
  atualizado_por TEXT NOT NULL,
  excluido_em TEXT,
  chave TEXT NOT NULL,
  valor TEXT NOT NULL CHECK (json_valid(valor))
);
CREATE UNIQUE INDEX preferencia_sincronizada_unica ON preferencia_sincronizada (chave) WHERE excluido_em IS NULL;

-- Tabelas locais (não sincronizam)

CREATE TABLE atalho (
  id TEXT PRIMARY KEY NOT NULL,
  escopo TEXT NOT NULL,
  combinacao TEXT NOT NULL UNIQUE
);

CREATE TABLE preferencia_local (
  chave TEXT PRIMARY KEY NOT NULL,
  valor TEXT NOT NULL CHECK (json_valid(valor))
);

CREATE TABLE historico_item (
  id TEXT PRIMARY KEY NOT NULL,
  criado_em TEXT NOT NULL,
  modo_id TEXT,
  modo_nome TEXT NOT NULL DEFAULT '',
  modo_emoji TEXT NOT NULL DEFAULT '',
  programa_plataforma TEXT,
  programa_identificador TEXT,
  programa_nome TEXT,
  duracao_audio_ms INTEGER NOT NULL DEFAULT 0,
  audio_arquivo TEXT,
  audio_apagado_em TEXT,
  estado TEXT NOT NULL CHECK (estado IN ('processando', 'colado', 'nao_colado', 'falhou', 'cancelado')),
  transcricao_provedor TEXT,
  transcricao_modelo TEXT,
  texto_transcrito TEXT NOT NULL DEFAULT '',
  texto_limpo TEXT NOT NULL DEFAULT '',
  texto_final TEXT NOT NULL DEFAULT '',
  refinamento_estado TEXT NOT NULL DEFAULT 'nao_aplicado' CHECK (refinamento_estado IN ('nao_aplicado', 'ok', 'falhou')),
  refinamento_provedor TEXT,
  refinamento_modelo TEXT,
  refinamento_motivo TEXT,
  tempo_transcricao_ms INTEGER,
  tempo_refinamento_ms INTEGER,
  tempo_total_ms INTEGER,
  contexto_selecao TEXT,
  contexto_area_transferencia TEXT,
  contexto_cortado INTEGER NOT NULL DEFAULT 0,
  contexto_aviso TEXT,
  erro_codigo TEXT,
  erro_mensagem TEXT
);
CREATE INDEX historico_item_criado_em ON historico_item (criado_em DESC, id DESC);

-- Busca de texto (RF-16.2), mantida por gatilhos.
CREATE VIRTUAL TABLE historico_busca USING fts5 (
  texto_transcrito, texto_final,
  content = 'historico_item', content_rowid = 'rowid'
);
CREATE TRIGGER historico_busca_ai AFTER INSERT ON historico_item BEGIN
  INSERT INTO historico_busca (rowid, texto_transcrito, texto_final) VALUES (new.rowid, new.texto_transcrito, new.texto_final);
END;
CREATE TRIGGER historico_busca_ad AFTER DELETE ON historico_item BEGIN
  INSERT INTO historico_busca (historico_busca, rowid, texto_transcrito, texto_final) VALUES ('delete', old.rowid, old.texto_transcrito, old.texto_final);
END;
CREATE TRIGGER historico_busca_au AFTER UPDATE ON historico_item BEGIN
  INSERT INTO historico_busca (historico_busca, rowid, texto_transcrito, texto_final) VALUES ('delete', old.rowid, old.texto_transcrito, old.texto_final);
  INSERT INTO historico_busca (rowid, texto_transcrito, texto_final) VALUES (new.rowid, new.texto_transcrito, new.texto_final);
END;

CREATE TABLE modelo_local (
  arquivo TEXT PRIMARY KEY NOT NULL,
  tamanho_bytes INTEGER NOT NULL,
  sha256 TEXT NOT NULL,
  estado TEXT NOT NULL DEFAULT 'nao_baixado' CHECK (estado IN ('nao_baixado', 'baixando', 'pausado', 'pronto')),
  bytes_baixados INTEGER NOT NULL DEFAULT 0
);

CREATE TABLE sincronizacao_estado (
  dispositivo_id TEXT PRIMARY KEY NOT NULL,
  nome_dispositivo TEXT NOT NULL,
  pasta TEXT,
  espaco_id TEXT,
  relogio TEXT,
  ultima_leitura_em TEXT,
  ultima_escrita_em TEXT,
  pendente INTEGER NOT NULL DEFAULT 0,
  ultimo_erro TEXT
);

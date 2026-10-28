-- Lista inicial de palavras de preenchimento em português e inglês (RF-09.2). Ids fixos: as
-- máquinas que migram sozinhas chegam às mesmas linhas, e a sincronização não duplica a lista.
-- Entradas com "…" só valem quando o texto traz as reticências (hesitação), ver limpeza/algoritmo.rs.
INSERT INTO palavra_preenchimento (id, atualizado_em, atualizado_por, palavra, palavra_normalizada) VALUES
  ('padrao-e-reticencias', '0', 'migracao', 'é…', 'é…'),
  ('padrao-ha', '0', 'migracao', 'hã', 'hã'),
  ('padrao-ahn', '0', 'migracao', 'ahn', 'ahn'),
  ('padrao-hum', '0', 'migracao', 'hum', 'hum'),
  ('padrao-ne', '0', 'migracao', 'né', 'né'),
  ('padrao-tipo', '0', 'migracao', 'tipo', 'tipo'),
  ('padrao-uh', '0', 'migracao', 'uh', 'uh'),
  ('padrao-uhm', '0', 'migracao', 'uhm', 'uhm'),
  ('padrao-um-reticencias', '0', 'migracao', 'um…', 'um…'),
  ('padrao-you-know', '0', 'migracao', 'you know', 'you know');

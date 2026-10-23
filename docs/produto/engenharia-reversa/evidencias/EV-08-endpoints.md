# EV-08 — Endpoints encontrados nas strings do binário

Origem: `strings -a -n 8` no executável principal do VoiceInk 2.13 (observação por ferramenta de shell, não é evidência do REA). Só host e caminho; query strings removidas; nenhum token.

## HTTPS
- `aistudio.google.com/app/apikey`
- `api.anthropic.com/v1/messages`
- `api.assemblyai.com/v2/transcript`
- `api.assemblyai.com/v2/upload`
- `api.cartesia.ai/voices`
- `api.cerebras.ai/v1/chat/completions`
- `api.deepgram.com/v1/listen`
- `api.deepgram.com/v1/projects`
- `api.elevenlabs.io/v1/speech-to-text`
- `api.elevenlabs.io/v1/user`
- `api.groq.com/openai`
- `api.groq.com/openai/v1/chat`
- `api.mistral.ai/v1/audio/transcriptions`
- `api.mistral.ai/v1/chat/completions`
- `api.mistral.ai/v1/models`
- `api.openai.com/v1/audio/transcriptions`
- `api.openai.com/v1/chat/completions`
- `api.openai.com/v1/models`
- `api.soniox.com/v1`
- `api.soniox.com/v1/files`
- `api.soniox.com/v1/transcriptions`
- `api.x.ai/v1/api-key`
- `api.x.ai/v1/stt`
- `asr.api.speechmatics.com/v2`
- `asr.api.speechmatics.com/v2/jobs`
- `beingpax.github.io/VoiceInk/announcements.json`
- `cloud.cerebras.ai/platform`
- `console.anthropic.com/settings/keys`
- `console.deepgram.com/project/keys`
- `console.groq.com/keys`
- `console.mistral.ai/api-keys`
- `console.soniox.com/api-keys`
- `console.speechmatics.com`
- `console.x.ai`
- `cookbook.openai.com/examples/whisper_prompting_guide`
- `discord.gg/xryDy57nYD`
- `elevenlabs.io/app/settings/api-keys`
- `generativelanguage.googleapis.com/upload/v1beta/files`
- `generativelanguage.googleapis.com/v1/interactions`
- `generativelanguage.googleapis.com/v1/models`
- `generativelanguage.googleapis.com/v1beta`
- `generativelanguage.googleapis.com/v1beta/interactions`
- `generativelanguage.googleapis.com/v1beta/models`
- `github.com/Beingpax/VoiceInk`
- `github.com/Beingpax/VoiceInk/releases`
- `github.com/FluidInference/FluidAudio/issues`
- `huggingface.co`
- `huggingface.co/ggerganov/whisper.cpp/resolve`
- `openrouter.ai/api/v1/chat`
- `openrouter.ai/api/v1/key`
- `openrouter.ai/api/v1/models`
- `openrouter.ai/keys`
- `platform.openai.com/api-keys`
- `play.cartesia.ai/keys`
- `polar.sh/beingpax/portal/request`
- `tryvoiceink.com/affiliate`
- `tryvoiceink.com/buy`
- `tryvoiceink.com/common-issues`
- `tryvoiceink.com/docs`
- `tryvoiceink.com/docs/custom-local-whisper-models`
- `tryvoiceink.com/docs/modes`
- `tryvoiceink.com/docs/recommended-models`
- `tryvoiceink.com/recommended-models`
- `www.assemblyai.com/dashboard/signup`
- `www.google.com/search`
- `www.youtube.com`

## WebSocket (transcrição em tempo real)
- `wss://api.cartesia.ai/stt/turns/websocket`
- `wss://api.deepgram.com/v1/listen`
- `wss://api.elevenlabs.io/v1/speech-to-text/realtime`
- `wss://api.mistral.ai/v1/audio/transcriptions/realtime`
- `wss://api.x.ai/v1/stt`
- `wss://eu2.rt.speechmatics.com/v2`
- `wss://generativelanguage.googleapis.com/ws/google.ai.generativelanguage.v1beta.GenerativeService.BidiGenerateContent`
- `wss://streaming.assemblyai.com/v3/ws`
- `wss://stt-rt.soniox.com/transcribe-websocket`

## Local
- `localhost:11434` (porta padrão do Ollama)

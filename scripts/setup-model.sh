#!/usr/bin/env bash
# Fait tourner un Luciole-1B-Instruct *généraliste* localement avec llama-server
# (macOS / Apple Silicon, Metal automatique). Lucid relaie ensuite vers ce serveur.
#
# Usage :
#   ./scripts/setup-model.sh                 # auto-détecte un GGUF local, port 8081
#   MODEL=/chemin/vers/modele.gguf ./scripts/setup-model.sh
#   PORT=8080 ./scripts/setup-model.sh
#
# NB : on écarte les variantes "actions"/"SFT" (routeurs d'intentions), qui
# émettent du JSON et ne conviennent PAS à la correction de texte.
set -euo pipefail

PORT="${PORT:-8081}"
MODEL="${MODEL:-}"

if [ -z "$MODEL" ]; then
  MODEL=$(find "$HOME" -iname "Luciole-1B-Instruct*Q8_0.gguf" 2>/dev/null | grep -viE "actions|sft" | head -1 || true)
  [ -z "$MODEL" ] && MODEL=$(find "$HOME" -iname "Luciole-1B-Instruct*Q4_K_M.gguf" 2>/dev/null | grep -viE "actions|sft" | head -1 || true)
fi

if [ -z "$MODEL" ] || [ ! -f "$MODEL" ]; then
  echo "Aucun GGUF Luciole-1B-Instruct généraliste trouvé."
  echo "Téléchargez-en un (Q8_0 recommandé) depuis :"
  echo "  https://huggingface.co/mmaudet/Luciole-1B-Instruct-GGUF"
  echo "puis relancez : MODEL=/chemin/vers/modele.gguf $0"
  exit 1
fi

if lsof -nP -iTCP:"$PORT" -sTCP:LISTEN >/dev/null 2>&1; then
  echo "Le port $PORT est déjà utilisé. Choisissez-en un autre : PORT=8082 $0"
  exit 1
fi

echo "Modèle  : $MODEL"
echo "Serveur : http://127.0.0.1:$PORT  (llama-server, Metal)"
echo "Ensuite : LUCID_BACKEND__BASE_URL=http://127.0.0.1:$PORT/v1 lucid serve"
echo "          (ou réglez backend.base_url dans config.toml)"
exec llama-server -m "$MODEL" -c 4096 -ngl 99 --host 127.0.0.1 --port "$PORT"

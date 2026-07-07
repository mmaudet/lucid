# Lucid

Correcteur de transcriptions de dictée **en français**, **100 % local**, exposant une API
compatible **OpenAI**. Lucid insère une étape de *réparation* légère entre le moteur de dictée
(VoiceInk, Handy, FluidVoice…) et le texte final : il corrige noms propres, patronymes,
toponymes, jargon, accents et ponctuation via **Luciole-1B-Instruct** servi localement par
`llama.cpp`, sans aucun aller-retour cloud.

> **État : incrément 1 (noyau headless).** Le serveur et le pipeline de correction fonctionnent
> de bout en bout en ligne de commande. L'app barre de menus, l'éditeur de dictionnaire, le
> journal SQLite et les statistiques viennent dans les incréments suivants (voir le PRD).

## Prérequis

- macOS (Apple Silicon), `llama.cpp` (`brew install llama.cpp`), Rust (`cargo`).
- Un GGUF **Luciole-1B-Instruct généraliste** (Q8_0 recommandé ; Q4_K_M pour RAM/vitesse).
  ⚠️ Pas une variante « actions »/« SFT » : ce sont des routeurs d'intentions qui renvoient du
  JSON, pas des correcteurs.

## Démarrage rapide

1. **Lancer le modèle** (llama-server) :
   ```bash
   ./scripts/setup-model.sh            # auto-détecte un GGUF local, port 8081
   # ou : MODEL=/chemin/vers/Luciole-1B-Instruct-Q8_0.gguf ./scripts/setup-model.sh
   ```

2. **Lancer Lucid** :
   ```bash
   cargo run -- serve
   # -> écoute sur http://127.0.0.1:8790/v1 et affiche le token bearer
   ```
   Si llama-server n'est pas sur `:8080`, pointez Lucid dessus :
   ```bash
   LUCID_BACKEND__BASE_URL=http://127.0.0.1:8081/v1 cargo run -- serve
   # ou réglez backend.base_url dans ~/Library/Application Support/Lucid/config.toml
   ```

3. **Vérifier** :
   ```bash
   curl -s http://127.0.0.1:8790/health
   TOKEN=... # affiché au démarrage
   curl -s http://127.0.0.1:8790/v1/chat/completions \
     -H "Authorization: Bearer $TOKEN" -H "Content-Type: application/json" \
     -d '{"model":"lucid","messages":[{"role":"user","content":"bonjour je mapelle jan dupont a lyon"}]}'
   # -> "Bonjour, je m'appelle Jean Dupont à Lyon."
   ```

## Intégration VoiceInk

Réglages d'**AI Enhancement** → provider **OpenAI-compatible personnalisé** :

| Champ | Valeur |
|---|---|
| Base URL | `http://127.0.0.1:8790/v1` |
| API key | le token bearer affiché au démarrage (ou `local` si l'auth est désactivée) |
| Model | `lucid` |

Même configuration (3 champs) pour **Handy** et **FluidVoice**.

## Dictionnaire

Ajoutez vos graphies métier dans `~/Library/Application Support/Lucid/dictionary.json` :
```json
{ "terms": [ { "canonical": "LINAGORA", "aliases": ["linagora", "lina gora"] } ] }
```
Lucid les injecte dans le prompt (« utilise ces graphies exactes »). Rechargé au prochain
démarrage du service (édition à chaud + UI : incrément M4).

## Configuration

`~/Library/Application Support/Lucid/config.toml` (créé au 1er lancement) ; surcharges via
variables `LUCID_*` (ex. `LUCID_SERVER__PORT=9000`). `cargo run -- doctor` affiche la config
résolue et teste la joignabilité du backend.

## Conception (correction)

- **prompt few-shot** en tours de chat + **stop sequences** : indispensable pour qu'un modèle 1B
  suive strictement l'instruction « renvoie uniquement le texte corrigé ».
- **Garde-fous / fail-safe** : sortie vide, trop longue, ou backend injoignable → le texte
  d'entrée est renvoyé **inchangé** (ne jamais dégrader la transcription).
- **Streaming** : la complétion est bufferisée puis ré-émise en SSE (préserve le fail-safe).

## Tests

```bash
cargo test                                           # suite complète (mock, déterministe)
cargo test --test nonreg_reel -- --ignored --nocapture   # non-régression FR contre le modèle réel
```

## Confidentialité

Écoute sur `127.0.0.1` uniquement, bearer activé par défaut, **aucune télémétrie**, aucun appel
réseau hors backend localhost.

//! Orchestration de la correction.
pub mod common_words;
pub mod guardrails;
pub mod prompt;

use crate::backends::{Backend, BackendRequest};
use crate::config::CorrectionConfig;
use crate::dictionary::Dictionary;
use crate::openai::{ChatMessage, ChatRequest};

pub use guardrails::{Outcome, Status};

/// Dernier message `user` = texte à corriger (balises d'enrobage retirées).
pub fn extract_input(messages: &[ChatMessage]) -> String {
    let raw = messages
        .iter()
        .rev()
        .find(|m| m.role == "user")
        .map(|m| m.content.clone())
        .unwrap_or_default();
    strip_wrappers(&raw)
}

/// Retire les balises d'enrobage ajoutées par certains clients (ex. VoiceInk enveloppe
/// la transcription dans <TRANSCRIPT>…</TRANSCRIPT>), qui perturbent un petit modèle.
fn strip_wrappers(s: &str) -> String {
    s.replace("<TRANSCRIPT>", "")
        .replace("</TRANSCRIPT>", "")
        .trim()
        .to_string()
}

/// Premier message `system` entrant (pour prompt_mode=prepend).
pub fn extract_system(messages: &[ChatMessage]) -> Option<String> {
    messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone())
}

/// Pipeline complet : prompt -> backend -> garde-fous -> fail-safe.
pub async fn correct(
    backend: &dyn Backend,
    dict: &Dictionary,
    cfg: &CorrectionConfig,
    req: &ChatRequest,
) -> Outcome {
    let input = extract_input(&req.messages);
    let incoming_system = extract_system(&req.messages);
    let dict_rendered = dict.render_for_prompt(cfg.dict_token_budget);
    let messages = prompt::build_messages(
        cfg.prompt_mode,
        incoming_system.as_deref(),
        &dict_rendered,
        &input,
    );

    let breq = BackendRequest {
        messages,
        temperature: req.temperature.unwrap_or(cfg.temperature),
        top_p: req.top_p.unwrap_or(cfg.top_p),
        max_tokens: req
            .max_tokens
            .unwrap_or_else(|| guardrails::compute_max_tokens(&input, cfg.max_output_ratio)),
        model: req.model.clone().unwrap_or_else(|| "luciole".into()),
        stop: cfg.stop.clone(),
    };

    let mut outcome = match backend.complete(&breq).await {
        Ok(output) => guardrails::evaluate(&input, &output, cfg.max_output_ratio),
        Err(_) => Outcome {
            text: input.trim().to_string(),
            status: Status::FailSafe,
        },
    };

    // Post-traitement déterministe : force les graphies canoniques du dictionnaire
    // (hors LLM). Appliqué MÊME en fail-safe : le stop "\n" peut vider une sortie et
    // déclencher le fail-safe ; on veut que les noms connus soient corrigés quoi qu'il arrive.
    let dicted = apply_dictionary(&outcome.text, dict);
    if dicted != outcome.text {
        outcome.text = dicted;
        if outcome.status == Status::Unchanged {
            outcome.status = Status::Corrected;
        }
    }
    outcome
}

/// Remplace, dans le texte, chaque variante du dictionnaire par sa graphie canonique
/// (insensible à la casse, uniquement sur des frontières de mot). Déterministe.
pub fn apply_dictionary(text: &str, dict: &Dictionary) -> String {
    let mut pairs: Vec<(&str, &str)> = Vec::new();
    for t in &dict.terms {
        for a in &t.aliases {
            if a.chars().count() < 3 {
                continue;
            }
            // Garde-fou : ne JAMAIS appliquer un alias mono-mot qui est un mot français
            // courant (« aime »→Aimé corromprait « il aime… »). Les alias multi-mots passent.
            if !a.contains(char::is_whitespace) && common_words::is_common_word(a) {
                continue;
            }
            pairs.push((a.as_str(), t.canonical.as_str()));
        }
    }
    // Variantes les plus longues d'abord (ex. « lina gora » avant « lina »).
    pairs.sort_by_key(|(a, _)| std::cmp::Reverse(a.chars().count()));

    let mut out = text.to_string();
    for (alias, canonical) in pairs {
        out = replace_word_ci(&out, alias, canonical);
    }
    out
}

/// Renvoie les alias « à risque » : mono-mot ET mot français courant (donc jamais
/// appliqués). Utilisé à l'édition du dictionnaire pour refuser un alias ambigu.
pub fn risky_aliases(dict: &Dictionary) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for t in &dict.terms {
        for a in &t.aliases {
            if !a.contains(char::is_whitespace) && common_words::is_common_word(a) {
                out.push((t.canonical.clone(), a.clone()));
            }
        }
    }
    out
}

/// Remplacement insensible à la casse aux frontières de mot. Sûr pour le français
/// (la minusculisation y préserve la longueur en octets) ; sinon renvoie l'entrée.
fn replace_word_ci(haystack: &str, needle: &str, replacement: &str) -> String {
    let hay_lower = haystack.to_lowercase();
    let need_lower = needle.to_lowercase();
    if need_lower.is_empty() || hay_lower.len() != haystack.len() {
        return haystack.to_string();
    }
    let mut result = String::new();
    let mut last = 0;
    let mut from = 0;
    while let Some(rel) = hay_lower[from..].find(&need_lower) {
        let start = from + rel;
        let end = start + need_lower.len();
        let before_ok = start == 0
            || !haystack[..start].chars().next_back().is_some_and(|c| c.is_alphanumeric());
        let after_ok = end >= haystack.len()
            || !haystack[end..].chars().next().is_some_and(|c| c.is_alphanumeric());
        if before_ok && after_ok {
            result.push_str(&haystack[last..start]);
            result.push_str(replacement);
            last = end;
            from = end;
        } else {
            from = start + need_lower.len().max(1);
        }
    }
    result.push_str(&haystack[last..]);
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dictionary::Dictionary;
    use crate::openai::ChatMessage;

    #[test]
    fn apply_dictionary_force_les_graphies() {
        let dict = Dictionary::from_json(
            r#"{"terms":[{"canonical":"Michel-Marie Maudet","aliases":["michel marie mode"]},{"canonical":"VoiceInk","aliases":["voice inque"]}]}"#,
        );
        assert_eq!(
            apply_dictionary("je vois voice inque et michel marie mode", &dict),
            "je vois VoiceInk et Michel-Marie Maudet"
        );
        assert_eq!(apply_dictionary("Voice Inque installé", &dict), "VoiceInk installé");
    }

    #[test]
    fn apply_dictionary_respecte_les_frontieres_de_mot() {
        let dict = Dictionary::from_json(r#"{"terms":[{"canonical":"Chat","aliases":["cat"]}]}"#);
        assert_eq!(apply_dictionary("the category", &dict), "the category");
        assert_eq!(apply_dictionary("un cat noir", &dict), "un Chat noir");
    }

    #[test]
    fn apply_dictionary_ignore_les_alias_ambigus() {
        // « aime » est un mot courant : jamais appliqué (sinon « il aime… » serait corrompu).
        let dict = Dictionary::from_json(r#"{"terms":[{"canonical":"Aimé","aliases":["aime"]}]}"#);
        assert_eq!(apply_dictionary("il aime le café", &dict), "il aime le café");
        assert_eq!(risky_aliases(&dict).len(), 1);
        // Un alias distinctif (avec apostrophe) passe.
        let d2 = Dictionary::from_json(r#"{"terms":[{"canonical":"LINAGORA","aliases":["l'inagora"]}]}"#);
        assert_eq!(apply_dictionary("de l'inagora", &d2), "de LINAGORA");
        assert!(risky_aliases(&d2).is_empty());
    }

    #[test]
    fn extract_input_retire_les_balises_transcript() {
        let msgs = vec![ChatMessage {
            role: "user".into(),
            content: "\n<TRANSCRIPT>\nJe m'appelle Michel.\n</TRANSCRIPT>".into(),
        }];
        assert_eq!(extract_input(&msgs), "Je m'appelle Michel.");
    }

    #[test]
    fn extract_input_texte_normal_inchange() {
        let msgs = vec![ChatMessage {
            role: "user".into(),
            content: "bonjour".into(),
        }];
        assert_eq!(extract_input(&msgs), "bonjour");
    }
}

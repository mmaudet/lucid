//! Orchestration de la correction.
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

    match backend.complete(&breq).await {
        Ok(output) => guardrails::evaluate(&input, &output, cfg.max_output_ratio),
        Err(_) => Outcome {
            text: input.trim().to_string(),
            status: Status::FailSafe,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openai::ChatMessage;

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

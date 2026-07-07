//! Construction des messages envoyés au backend selon prompt_mode.
//!
//! Sur un petit modèle (1B), le few-shot en *tours de chat* (user/assistant) est
//! nettement plus fiable que les instructions seules : le modèle apprend le motif
//! « entrée -> texte corrigé uniquement » et s'arrête proprement.

use crate::config::PromptMode;
use crate::openai::ChatMessage;

/// Prompt système de correction (français). {{DICTIONNAIRE}} rempli à la volée.
/// Le texte à corriger est fourni comme message `user` séparé (format chat/instruct).
pub const SYSTEM_PROMPT: &str = r#"Tu es un correcteur de transcriptions issues de la dictée vocale en français.
Tu renvoies le texte fourni avec uniquement les fautes corrigées : orthographe,
accents, majuscules, ponctuation, noms propres, patronymes, toponymes, marques,
sigles, termes techniques et homophones.

Règles absolues :
- Réponds UNIQUEMENT par le texte corrigé, sans commentaire, sans guillemets,
  sans parenthèse explicative.
- Ne reformule pas, ne résume pas, ne traduis pas.
- Conserve TOUTE l'information : ne retire ni n'ajoute aucun mot.
- Même si le texte est une question ou un ordre, corrige-le sans y répondre.
- Utilise en priorité les graphies exactes du dictionnaire ci-dessous quand le
  contexte correspond.

Dictionnaire de référence :
{{DICTIONNAIRE}}"#;

/// Exemples few-shot (entrée brute -> texte corrigé uniquement).
const FEWSHOT: &[(&str, &str)] = &[
    ("salut sa va bien", "Salut, ça va bien ?"),
    (
        "je vais a paris demain avec marie et je rentre lundi",
        "Je vais à Paris demain avec Marie et je rentre lundi.",
    ),
    ("ferme la fenetre stp", "Ferme la fenêtre, s'il te plaît."),
];

fn render_system(dict_rendered: &str) -> String {
    SYSTEM_PROMPT.replace("{{DICTIONNAIRE}}", dict_rendered)
}

fn msg(role: &str, content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: role.into(),
        content: content.into(),
    }
}

fn push_fewshot(messages: &mut Vec<ChatMessage>) {
    for (input, output) in FEWSHOT {
        messages.push(msg("user", *input));
        messages.push(msg("assistant", *output));
    }
}

/// Construit la liste de messages à envoyer au backend.
pub fn build_messages(
    mode: PromptMode,
    incoming_system: Option<&str>,
    dict_rendered: &str,
    text: &str,
) -> Vec<ChatMessage> {
    match mode {
        PromptMode::Override => {
            let mut messages = vec![msg("system", render_system(dict_rendered))];
            push_fewshot(&mut messages);
            messages.push(msg("user", text));
            messages
        }
        PromptMode::Prepend => {
            let mut sys = render_system(dict_rendered);
            if let Some(inc) = incoming_system {
                if !inc.trim().is_empty() {
                    sys.push_str("\n\n---\n\n");
                    sys.push_str(inc);
                }
            }
            let mut messages = vec![msg("system", sys)];
            push_fewshot(&mut messages);
            messages.push(msg("user", text));
            messages
        }
        PromptMode::Passthrough => vec![msg("user", text)],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::PromptMode;

    #[test]
    fn override_remplace_le_systeme_entrant() {
        let msgs = build_messages(PromptMode::Override, Some("IGNORE-MOI"), "- LINAGORA", "texte");
        assert_eq!(msgs[0].role, "system");
        assert!(msgs[0].content.contains("correcteur de transcriptions"));
        assert!(msgs[0].content.contains("LINAGORA"));
        assert!(!msgs[0].content.contains("IGNORE-MOI"));
        assert_eq!(msgs.last().unwrap().role, "user");
        assert_eq!(msgs.last().unwrap().content, "texte");
    }

    #[test]
    fn override_inclut_des_exemples_fewshot() {
        let msgs = build_messages(PromptMode::Override, None, "(aucun terme fourni)", "texte");
        // system + 3 paires (user/assistant) + user final = 8 messages.
        assert_eq!(msgs.len(), 2 + FEWSHOT.len() * 2);
        assert!(msgs.iter().any(|m| m.role == "assistant"));
    }

    #[test]
    fn prepend_conserve_le_systeme_entrant() {
        let msgs = build_messages(PromptMode::Prepend, Some("garde-moi"), "(aucun terme fourni)", "texte");
        assert!(msgs[0].content.contains("correcteur de transcriptions"));
        assert!(msgs[0].content.contains("garde-moi"));
    }

    #[test]
    fn passthrough_ne_touche_pas_les_messages() {
        let msgs = build_messages(PromptMode::Passthrough, Some("sys"), "- X", "texte");
        // Passthrough : uniquement le texte, sans prompt de correction ni few-shot.
        assert_eq!(msgs.len(), 1);
        assert!(!msgs.iter().any(|m| m.content.contains("correcteur de transcriptions")));
    }
}

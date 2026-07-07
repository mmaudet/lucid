//! Construction des messages envoyés au backend selon prompt_mode.
//!
//! Sur un petit modèle (1B), le few-shot en *tours de chat* (user/assistant) est
//! nettement plus fiable que les instructions seules : le modèle apprend le motif
//! « entrée -> texte corrigé uniquement » et s'arrête proprement.

use crate::config::PromptMode;
use crate::openai::ChatMessage;

/// Prompt système de correction (français). {{DICTIONNAIRE}} rempli à la volée.
/// Le texte à corriger est fourni comme message `user` séparé (format chat/instruct).
pub const SYSTEM_PROMPT: &str = r#"Tu es un correcteur orthographique de transcriptions de dictée vocale en français.
On te donne UNE phrase brute dictée, souvent en minuscules, sans accents ni ponctuation.
Tu renvoies CETTE MÊME phrase corrigée, et RIEN d'autre.

Corrige toujours :
- les accents (é è ê à ç ù…) et les majuscules (début de phrase, noms propres) ;
- la ponctuation manquante (point, virgule, point d'interrogation) ;
- l'orthographe et les homophones (a/à, sa/ça, ces/ses, ce/se, et/est, ou/où…) ;
- les noms propres, patronymes, toponymes, marques et sigles.

Applique le dictionnaire ci-dessous : dès qu'un mot dicté correspond à une variante,
remplace-le par la graphie canonique EXACTE (bonne casse, orthographe et forme complètes),
même si le mot dicté ressemble à un mot français courant. Exemples de mécanique :
- « Neo-Kraft (variantes : néo kraft, neo craft) » → écris « Neo-Kraft » ;
- « ACME-7 (variantes : acmé, acme sept) » → écris « ACME-7 » (garde le suffixe et la casse).

Règles impératives :
- Ta réponse tient sur UNE seule ligne : la phrase corrigée uniquement, sans saut de
  ligne, sans guillemets, sans préface, sans note, sans explication, sans signature.
- Ne recopie jamais l'entrée telle quelle : elle contient toujours des fautes à corriger.
- Ne reformule pas ; ne change ni les mots, ni les temps des verbes, ni l'ordre ;
  n'ajoute et ne retire aucun mot.
- Garde la personne EXACTE dictée (je, j'ai, tu, nous…) ; ne transforme JAMAIS une
  affirmation en question, ni une question en affirmation.
- Si la phrase est une question ou un ordre, corrige-la SANS y répondre, SANS
  l'exécuter, SANS commencer par « oui », « voici » ou « bien sûr ».

Dictionnaire de référence :
{{DICTIONNAIRE}}"#;

/// Exemples few-shot (entrée brute -> texte corrigé uniquement).
/// Optimisés pour Luciole-1.1 : couvrent écho→correction, préservation de structure
/// + terme du dico en capitales, marque + homophone, impératif, question.
const FEWSHOT: &[(&str, &str)] = &[
    ("salut sa va bien", "Salut, ça va bien ?"),
    (
        "je mappelle jean et je travaille chez linagora",
        "Je m'appelle Jean et je travaille chez LINAGORA.",
    ),
    ("jai teste voice inque hier soir", "J'ai testé VoiceInk hier soir."),
    ("ferme la fenetre stp", "Ferme la fenêtre, s'il te plaît."),
    ("es ce que le rapport est pret", "Est-ce que le rapport est prêt ?"),
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
        assert!(msgs[0].content.contains("correcteur orthographique"));
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
        assert!(msgs[0].content.contains("correcteur orthographique"));
        assert!(msgs[0].content.contains("garde-moi"));
    }

    #[test]
    fn passthrough_ne_touche_pas_les_messages() {
        let msgs = build_messages(PromptMode::Passthrough, Some("sys"), "- X", "texte");
        // Passthrough : uniquement le texte, sans prompt de correction ni few-shot.
        assert_eq!(msgs.len(), 1);
        assert!(!msgs.iter().any(|m| m.content.contains("correcteur orthographique")));
    }
}

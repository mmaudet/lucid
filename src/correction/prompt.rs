//! Construction des messages envoyés au backend selon prompt_mode.

use crate::config::PromptMode;
use crate::openai::ChatMessage;

/// Prompt système de correction (français). {{DICTIONNAIRE}} rempli à la volée.
/// Le texte à corriger est fourni comme message `user` séparé (format chat/instruct).
pub const SYSTEM_PROMPT: &str = r#"Tu es un correcteur de transcriptions issues de la dictée vocale en français.
Ta seule tâche : renvoyer le texte fourni en corrigeant les erreurs de
transcription, en particulier les noms propres, patronymes, noms de lieux,
marques, sigles et termes techniques, ainsi que les homophones, la casse,
les accents et la ponctuation évidente.

Règles strictes :
- Ne reformule pas, ne résume pas, ne traduis pas.
- N'ajoute et ne retire aucune information ; ne réponds pas au contenu.
- Le texte est en français ; conserve la langue.
- Utilise en priorité les graphies exactes du dictionnaire ci-dessous quand
  le contexte correspond.
- En cas de doute sur un nom absent du dictionnaire, choisis la graphie la
  plus plausible sans rien inventer.
- Réponds UNIQUEMENT par le texte corrigé, sans guillemets ni commentaire.

Dictionnaire de référence :
{{DICTIONNAIRE}}"#;

fn render_system(dict_rendered: &str) -> String {
    SYSTEM_PROMPT.replace("{{DICTIONNAIRE}}", dict_rendered)
}

/// Construit la liste de messages à envoyer au backend.
pub fn build_messages(
    mode: PromptMode,
    incoming_system: Option<&str>,
    dict_rendered: &str,
    text: &str,
) -> Vec<ChatMessage> {
    let user = ChatMessage { role: "user".into(), content: text.to_string() };
    match mode {
        PromptMode::Override => vec![
            ChatMessage { role: "system".into(), content: render_system(dict_rendered) },
            user,
        ],
        PromptMode::Prepend => {
            let mut sys = render_system(dict_rendered);
            if let Some(inc) = incoming_system {
                if !inc.trim().is_empty() {
                    sys.push_str("\n\n---\n\n");
                    sys.push_str(inc);
                }
            }
            vec![ChatMessage { role: "system".into(), content: sys }, user]
        }
        PromptMode::Passthrough => vec![user],
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
    fn prepend_conserve_le_systeme_entrant() {
        let msgs = build_messages(PromptMode::Prepend, Some("garde-moi"), "(aucun terme fourni)", "texte");
        assert!(msgs[0].content.contains("correcteur de transcriptions"));
        assert!(msgs[0].content.contains("garde-moi"));
    }

    #[test]
    fn passthrough_ne_touche_pas_les_messages() {
        let msgs = build_messages(PromptMode::Passthrough, Some("sys"), "- X", "texte");
        // Passthrough : uniquement le texte, sans prompt de correction.
        assert!(!msgs.iter().any(|m| m.content.contains("correcteur de transcriptions")));
    }
}

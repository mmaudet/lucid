//! Dictionnaire de corrections en lecture seule (édition différée à M4).

use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Dictionary {
    #[serde(default)]
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Term {
    pub canonical: String,
    #[serde(default)]
    pub aliases: Vec<String>,
}

impl Dictionary {
    /// Charge depuis le disque ; fichier absent ou invalide -> dictionnaire vide.
    pub fn load(path: &Path) -> Dictionary {
        match std::fs::read_to_string(path) {
            Ok(s) => Dictionary::from_json(&s),
            Err(_) => Dictionary::default(),
        }
    }

    pub fn from_json(s: &str) -> Dictionary {
        serde_json::from_str(s).unwrap_or_default()
    }

    /// Rend les graphies pour l'emplacement {{DICTIONNAIRE}} du prompt,
    /// borné par un budget de tokens estimé (~ 1 token / 3 caractères).
    pub fn render_for_prompt(&self, token_budget: usize) -> String {
        if self.terms.is_empty() {
            return "(aucun terme fourni)".to_string();
        }
        let mut out = String::new();
        let mut used = 0usize;
        for t in &self.terms {
            let line = if t.aliases.is_empty() {
                format!("- {}\n", t.canonical)
            } else {
                format!("- {} (variantes : {})\n", t.canonical, t.aliases.join(", "))
            };
            let cost = line.chars().count().div_ceil(3);
            if used + cost > token_budget {
                break;
            }
            used += cost;
            out.push_str(&line);
        }
        if out.is_empty() {
            "(aucun terme fourni)".to_string()
        } else {
            out.trim_end().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_et_rend_les_termes() {
        let json = r#"{"terms":[{"canonical":"LINAGORA","aliases":["linagora","lina gora"]}]}"#;
        let d = Dictionary::from_json(json);
        assert_eq!(d.terms.len(), 1);
        let rendu = d.render_for_prompt(256);
        assert!(rendu.contains("LINAGORA"));
        assert!(rendu.contains("lina gora"));
    }

    #[test]
    fn dictionnaire_vide_rend_placeholder() {
        let d = Dictionary::default();
        assert_eq!(d.render_for_prompt(256), "(aucun terme fourni)");
    }

    #[test]
    fn budget_tronque_la_liste() {
        let mut terms = Vec::new();
        for i in 0..1000 {
            terms.push(Term { canonical: format!("Terme{i}"), aliases: vec![] });
        }
        let d = Dictionary { terms };
        let rendu = d.render_for_prompt(50); // petit budget
        // Tronqué : bien moins que 1000 lignes.
        assert!(rendu.lines().count() < 100);
    }
}

//! Garde-fous de longueur + décision de fail-safe.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Corrected,
    Unchanged,
    FailSafe,
}

#[derive(Debug, Clone)]
pub struct Outcome {
    pub text: String,
    pub status: Status,
}

/// Évalue la sortie du backend ; renvoie l'entrée inchangée si non fiable.
pub fn evaluate(input: &str, output: &str, max_ratio: f32) -> Outcome {
    let trimmed = output.trim();
    let input_trimmed = input.trim();

    if trimmed.is_empty() {
        return failsafe(input_trimmed);
    }
    // Ratio de longueur (en caractères). Entrée vide -> on saute le contrôle.
    let in_len = input_trimmed.chars().count();
    if in_len > 0 {
        let out_len = trimmed.chars().count();
        if (out_len as f32) > (in_len as f32) * max_ratio {
            return failsafe(input_trimmed);
        }
    }
    let status = if trimmed == input_trimmed {
        Status::Unchanged
    } else {
        Status::Corrected
    };
    Outcome { text: trimmed.to_string(), status }
}

fn failsafe(input_trimmed: &str) -> Outcome {
    Outcome { text: input_trimmed.to_string(), status: Status::FailSafe }
}

/// Plafond de génération, relatif à la longueur d'entrée (estimation ~1 token / 3 car.).
pub fn compute_max_tokens(input: &str, ratio: f32) -> u32 {
    let est_in = (input.chars().count().max(1)).div_ceil(3) as f32;
    let val = (est_in * ratio).ceil() as u32 + 64;
    val.clamp(64, 2048)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sortie_vide_declenche_failsafe() {
        let o = evaluate("bonjour michel", "   ", 2.0);
        assert_eq!(o.status, Status::FailSafe);
        assert_eq!(o.text, "bonjour michel");
    }

    #[test]
    fn sortie_trop_longue_declenche_failsafe() {
        let o = evaluate("court", &"x".repeat(100), 2.0);
        assert_eq!(o.status, Status::FailSafe);
        assert_eq!(o.text, "court");
    }

    #[test]
    fn correction_normale_est_corrected() {
        let o = evaluate("michel marie mode", "Michel-Marie Maudet", 2.0);
        assert_eq!(o.status, Status::Corrected);
        assert_eq!(o.text, "Michel-Marie Maudet");
    }

    #[test]
    fn sortie_identique_est_unchanged() {
        let o = evaluate("déjà correct", "déjà correct", 2.0);
        assert_eq!(o.status, Status::Unchanged);
    }

    #[test]
    fn max_tokens_croit_avec_l_entree() {
        assert!(compute_max_tokens(&"a".repeat(300), 2.0) > compute_max_tokens("a", 2.0));
    }
}

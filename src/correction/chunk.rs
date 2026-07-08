//! Découpage d'un texte long en phrases (pour corriger chaque phrase séparément :
//! une petite tâche est fiable, un 1B dérive en « mode chatbot » sur un texte long).

/// Une phrase + le séparateur d'origine qui la suit (espaces / sauts de ligne),
/// réinjecté au recollage pour préserver la structure (paragraphes).
#[derive(Debug, Clone, PartialEq)]
pub struct Sentence {
    pub text: String,
    pub sep: String,
}

/// Découpe en phrases sur `. ! ? …` suivis d'une espace, et sur les sauts de ligne.
/// Ne coupe pas une décimale (`3.14`). Un séparateur seul (lignes vides) est rattaché
/// à la phrase précédente.
pub fn split_sentences(text: &str) -> Vec<Sentence> {
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Sentence> = Vec::new();
    let mut body = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];

        if c == '\n' {
            let mut sep = String::new();
            while i < chars.len() && chars[i].is_whitespace() {
                sep.push(chars[i]);
                i += 1;
            }
            push_unit(&mut out, &mut body, sep);
            continue;
        }

        body.push(c);
        let is_term = matches!(c, '.' | '!' | '?' | '…');
        let next_ws = chars.get(i + 1).map_or(true, |n| n.is_whitespace());
        let prev_digit = body.chars().rev().nth(1).is_some_and(|p| p.is_ascii_digit());
        let next_digit = chars.get(i + 1).is_some_and(|n| n.is_ascii_digit());
        let is_decimal = c == '.' && prev_digit && next_digit;

        if is_term && next_ws && !is_decimal {
            let mut sep = String::new();
            i += 1;
            while i < chars.len() && chars[i].is_whitespace() {
                sep.push(chars[i]);
                i += 1;
            }
            push_unit(&mut out, &mut body, sep);
            continue;
        }
        i += 1;
    }
    if !body.trim().is_empty() {
        out.push(Sentence {
            text: body.trim().to_string(),
            sep: String::new(),
        });
    }
    out
}

fn push_unit(out: &mut Vec<Sentence>, body: &mut String, sep: String) {
    let t = body.trim().to_string();
    body.clear();
    if t.is_empty() {
        if let Some(last) = out.last_mut() {
            last.sep.push_str(&sep);
        }
    } else {
        out.push(Sentence { text: t, sep });
    }
}

/// Recolle les phrases corrigées en réinjectant les séparateurs (préserve les
/// paragraphes : 1–2 sauts de ligne ; sinon une espace simple entre phrases).
pub fn rejoin(units: &[Sentence], corrected: &[String]) -> String {
    let mut out = String::new();
    for (i, c) in corrected.iter().enumerate() {
        out.push_str(c.trim());
        let sep = units.get(i).map(|u| u.sep.as_str()).unwrap_or("");
        if sep.contains('\n') {
            for _ in 0..sep.matches('\n').count().min(2) {
                out.push('\n');
            }
        } else if i + 1 < corrected.len() {
            out.push(' ');
        }
    }
    out.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoupe_en_phrases() {
        let u = split_sentences("Bonjour Jean. Comment vas-tu ? Très bien !");
        assert_eq!(u.len(), 3);
        assert_eq!(u[0].text, "Bonjour Jean.");
        assert_eq!(u[1].text, "Comment vas-tu ?");
        assert_eq!(u[2].text, "Très bien !");
    }

    #[test]
    fn ne_coupe_pas_les_decimales() {
        let u = split_sentences("Le taux est de 3.14 pour cent.");
        assert_eq!(u.len(), 1);
    }

    #[test]
    fn preserve_les_paragraphes() {
        let u = split_sentences("Phrase une.\n\nPhrase deux.");
        assert_eq!(u.len(), 2);
        assert!(u[0].sep.contains('\n'));
        let out = rejoin(&u, &["Phrase une.".into(), "Phrase deux.".into()]);
        assert_eq!(out, "Phrase une.\n\nPhrase deux.");
    }

    #[test]
    fn recolle_avec_espaces() {
        let u = split_sentences("Un. Deux. Trois.");
        let out = rejoin(&u, &["Un.".into(), "Deux.".into(), "Trois.".into()]);
        assert_eq!(out, "Un. Deux. Trois.");
    }

    #[test]
    fn texte_sans_ponctuation_reste_entier() {
        let u = split_sentences("juste une phrase sans point");
        assert_eq!(u.len(), 1);
        assert_eq!(u[0].text, "juste une phrase sans point");
    }
}

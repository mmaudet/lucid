//! Dictionnaire de corrections : chargement, rendu pour le prompt, et handle
//! partagé `DictionaryStore` (lecture lock-free via ArcSwap + persistance atomique).

use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Dictionary {
    #[serde(default)]
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Handle partagé du dictionnaire : lecture lock-free (ArcSwap) sur le hot-path
/// de correction, remplacement atomique persisté pour l'édition (M4), rechargement
/// à chaud. Le MÊME `DictionaryStore` est porté à travers les redémarrages du serveur
/// (voir ServerManager) pour que les éditions continuent de se propager.
pub struct DictionaryStore {
    inner: ArcSwap<Dictionary>,
    path: Option<PathBuf>,
}

impl DictionaryStore {
    /// Charge depuis le disque ; le chemin est mémorisé pour la persistance.
    pub fn load(path: &Path) -> Self {
        DictionaryStore {
            inner: ArcSwap::from_pointee(Dictionary::load(path)),
            path: Some(path.to_path_buf()),
        }
    }

    /// Sans persistance (tests / dev).
    pub fn in_memory(dict: Dictionary) -> Self {
        DictionaryStore {
            inner: ArcSwap::from_pointee(dict),
            path: None,
        }
    }

    /// Lecture lock-free du dictionnaire courant (hot-path correction).
    pub fn snapshot(&self) -> Arc<Dictionary> {
        self.inner.load_full()
    }

    /// Remplace le dictionnaire courant et le persiste atomiquement (si chemin connu).
    pub fn replace(&self, dict: Dictionary) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            persist_atomic(path, &dict)?;
        }
        self.inner.store(Arc::new(dict));
        Ok(())
    }

    /// Recharge depuis le disque (si chemin connu) ; sinon no-op.
    pub fn reload(&self) -> std::io::Result<()> {
        if let Some(path) = &self.path {
            self.inner.store(Arc::new(Dictionary::load(path)));
        }
        Ok(())
    }
}

/// Écriture atomique : fichier temporaire puis rename.
fn persist_atomic(path: &Path, dict: &Dictionary) -> std::io::Result<()> {
    let json = serde_json::to_string_pretty(dict)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
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
        assert!(rendu.lines().count() < 100);
    }

    #[test]
    fn store_in_memory_snapshot_et_replace() {
        let s = DictionaryStore::in_memory(Dictionary::default());
        assert!(s.snapshot().terms.is_empty());
        s.replace(Dictionary::from_json(
            r#"{"terms":[{"canonical":"LINAGORA","aliases":[]}]}"#,
        ))
        .unwrap();
        assert_eq!(s.snapshot().terms.len(), 1);
        assert_eq!(s.snapshot().terms[0].canonical, "LINAGORA");
    }

    #[test]
    fn store_persiste_et_recharge_depuis_disque() {
        let path = std::env::temp_dir().join(format!("lucid_test_dict_{}.json", std::process::id()));
        let _ = std::fs::remove_file(&path);
        let s = DictionaryStore::load(&path); // absent -> vide
        assert!(s.snapshot().terms.is_empty());
        s.replace(Dictionary::from_json(
            r#"{"terms":[{"canonical":"VoiceInk","aliases":["voice inque"]}]}"#,
        ))
        .unwrap();
        // Un nouveau store relit ce qui a été persisté.
        let s2 = DictionaryStore::load(&path);
        assert_eq!(s2.snapshot().terms.len(), 1);
        assert_eq!(s2.snapshot().terms[0].canonical, "VoiceInk");
        let _ = std::fs::remove_file(&path);
    }
}

//! Mots français courants — garde-fou contre les alias de dictionnaire ambigus.
//! Un alias mono-mot présent ici ne sera JAMAIS appliqué (sinon il corromprait du
//! texte normal : « aime »→Aimé, « car »→Caron, « mode »→Maudet…).

use std::collections::HashSet;
use std::sync::OnceLock;

/// Replie une chaîne pour la comparaison : minuscule + accents français retirés.
pub fn fold(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| match c {
            'à' | 'â' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'î' | 'ï' => 'i',
            'ô' | 'ö' => 'o',
            'û' | 'ü' | 'ù' => 'u',
            'ç' => 'c',
            'ÿ' => 'y',
            c => c,
        })
        .collect()
}

const COMMON: &str = "\
le la les un une de des du au aux et ou mais donc or ni car que qui quoi dont quand comme si \
je tu il elle on nous vous ils elles me te se lui leur moi toi soi eux \
mon ton son ma ta sa mes tes ses notre votre nos vos leurs \
ce cet cette ces celui celle ceux celles ceci cela \
dans par pour sur sous avec sans chez vers entre depuis pendant avant apres contre selon \
est sont etait etaient sera seront suis es sommes etes etre \
ai as ont avait avaient aura avoir eu \
fait faire fais font faisait dit dire va vais vont aller allait \
peut peux pouvons pouvez pouvoir veut veux voulons voulez vouloir \
doit dois devons devez devoir sait sais savons savez savoir \
voit vois voyons voyez voir prend prends prenons prendre \
vient viens venons venir met mets mettre \
aime aimes aiment aimait aide aides aident aidait \
pose poses posait posaient posons posez posent poser posee posees \
oui non pas plus moins tres bien mal peu beaucoup trop assez \
tout tous toute toutes rien seul seule meme autre autres \
grand grande petit petite bon bonne nouveau nouvelle premier derniere \
alors ensuite puis aussi encore deja ici voila voici maintenant enfin \
jour jours temps chose choses fois homme femme enfant enfants monde vie \
main pays eau an ans annee annees gens part cas point points \
note notes date dates mode heure heures semaine semaines mois \
projet projets travail reunion reunions message messages probleme problemes \
question questions reponse reponses dossier dossiers rapport rapports \
equipe equipes client clients service services produit produits \
code codes fichier fichiers systeme donnees valeur valeurs valide \
cadre nom noms numero mail mails compte comptes \
zero deux trois quatre cinq six sept huit neuf dix cent mille \
lundi mardi mercredi jeudi vendredi samedi dimanche \
janvier fevrier mars avril juin juillet aout septembre octobre novembre decembre \
name time made and are the you for with this that \
merci bonjour salut madame monsieur";

pub fn is_common_word(w: &str) -> bool {
    static SET: OnceLock<HashSet<String>> = OnceLock::new();
    let set = SET.get_or_init(|| COMMON.split_whitespace().map(fold).collect());
    set.contains(&fold(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detecte_les_mots_courants_dangereux() {
        for w in ["car", "aime", "aide", "mode", "notes", "enfants", "toi", "cadre", "valeur", "name", "voilà", "Même", "made"] {
            assert!(is_common_word(w), "« {w} » devrait être détecté comme mot courant");
        }
    }

    #[test]
    fn ne_flague_pas_les_noms_distinctifs() {
        for w in ["linagora", "twake", "onyoffice", "loiselet", "benoit", "dinum", "raphael", "tosit", "bellamy"] {
            assert!(!is_common_word(w), "« {w} » ne devrait PAS être flagué");
        }
    }
}

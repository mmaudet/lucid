//! Fonctions pures pour la supervision optionnelle du backend.
//! Le lancement/arrêt effectif du process vivra dans le ServerManager (Vague 1) ;
//! ici on ne garde que la logique déterministe et testable.

use crate::config::BackendConfig;

/// Extrait (host, port) d'une base_url `http(s)://host:port/...`.
pub fn parse_host_port(base_url: &str) -> Option<(String, u16)> {
    let s = base_url.split("://").nth(1).unwrap_or(base_url);
    let authority = s.split('/').next().unwrap_or(s);
    let (host, port) = authority.rsplit_once(':')?;
    Some((host.to_string(), port.parse().ok()?))
}

/// Déduit la commande de lancement du backend `(commande, args)`.
/// Priorité à `launch_command` explicite ; sinon déduction par `kind`.
pub fn derive_launch(cfg: &BackendConfig) -> Option<(String, Vec<String>)> {
    if let Some(cmd) = &cfg.launch_command {
        return Some((cmd.clone(), cfg.launch_args.clone()));
    }
    let (_host, port) = parse_host_port(&cfg.base_url)?;
    match cfg.kind.as_str() {
        "ollama" => Some(("ollama".into(), vec!["serve".into()])),
        "llamacpp" => {
            let model = cfg.model_path.clone()?;
            Some((
                "llama-server".into(),
                vec![
                    "-m".into(),
                    model,
                    "-c".into(),
                    "4096".into(),
                    "-ngl".into(),
                    "99".into(),
                    "--host".into(),
                    "127.0.0.1".into(),
                    "--port".into(),
                    port.to_string(),
                ],
            ))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn parse_host_port_ok() {
        assert_eq!(parse_host_port("http://127.0.0.1:8080/v1"), Some(("127.0.0.1".into(), 8080)));
        assert_eq!(parse_host_port("http://localhost:11434/v1"), Some(("localhost".into(), 11434)));
        assert_eq!(parse_host_port("pas-une-url"), None);
    }

    #[test]
    fn derive_launch_ollama() {
        let mut cfg = Config::default().backend;
        cfg.kind = "ollama".into();
        cfg.base_url = "http://127.0.0.1:11434/v1".into();
        let (cmd, args) = derive_launch(&cfg).unwrap();
        assert_eq!(cmd, "ollama");
        assert_eq!(args, vec!["serve".to_string()]);
    }

    #[test]
    fn derive_launch_llamacpp_avec_modele() {
        let mut cfg = Config::default().backend; // kind llamacpp, port 8080
        cfg.model_path = Some("/models/luciole-q8.gguf".into());
        let (cmd, args) = derive_launch(&cfg).unwrap();
        assert_eq!(cmd, "llama-server");
        assert!(args.contains(&"/models/luciole-q8.gguf".to_string()));
        assert!(args.contains(&"8080".to_string()));
    }

    #[test]
    fn derive_launch_llamacpp_sans_modele_none() {
        let cfg = Config::default().backend; // model_path None
        assert!(derive_launch(&cfg).is_none());
    }

    #[test]
    fn launch_command_explicite_prioritaire() {
        let mut cfg = Config::default().backend;
        cfg.launch_command = Some("mon-serveur".into());
        cfg.launch_args = vec!["--foo".into()];
        let (cmd, args) = derive_launch(&cfg).unwrap();
        assert_eq!(cmd, "mon-serveur");
        assert_eq!(args, vec!["--foo".to_string()]);
    }
}

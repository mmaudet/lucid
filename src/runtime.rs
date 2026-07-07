//! ServerManager : propriétaire UNIQUE du cycle de vie du serveur HTTP.
//!
//! Ungated (tokio/axum pur, sans Tauri) donc testable en TDD. Réutilisé à la fois
//! par le CLI `lucid serve` et par l'app Tauri (Vague 1). Détient la Config courante,
//! le backend, le `Arc<DictionaryStore>` et le `Store` — et les PORTE À L'IDENTIQUE
//! à travers les redémarrages (jamais rechargés du disque), pour que les éditions
//! dictionnaire/journal continuent de se propager après un restart de config.

use crate::backends::{self, Backend, BackendHealth};
use crate::config::{self, Config};
use crate::dictionary::{Dictionary, DictionaryStore};
use crate::server::{build_app, AppState};
use crate::store::Store;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

pub struct ServerManager {
    config: Config,
    backend: Arc<dyn Backend>,
    dictionary: Arc<DictionaryStore>,
    store: Store,
    running: Option<RunningServer>,
}

struct RunningServer {
    addr: SocketAddr,
    shutdown: oneshot::Sender<()>,
    handle: JoinHandle<()>,
}

impl ServerManager {
    /// Construit depuis la config (charge dictionnaire + journal comme en production).
    pub fn new(config: Config) -> Self {
        let backend = backends::from_config(&config.backend);
        let dictionary = Arc::new(match config::dictionary_path() {
            Ok(p) => DictionaryStore::load(&p),
            Err(_) => DictionaryStore::in_memory(Dictionary::default()),
        });
        let store = if config.journal.enabled {
            config::journal_path()
                .ok()
                .and_then(|p| Store::open(&p, &config.journal).ok())
                .unwrap_or_else(Store::disabled)
        } else {
            Store::disabled()
        };
        Self::with_parts(config, backend, dictionary, store)
    }

    /// Injection explicite des composants (tests / contrôle fin).
    pub fn with_parts(
        config: Config,
        backend: Arc<dyn Backend>,
        dictionary: Arc<DictionaryStore>,
        store: Store,
    ) -> Self {
        ServerManager { config, backend, dictionary, store, running: None }
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn dictionary(&self) -> &Arc<DictionaryStore> {
        &self.dictionary
    }

    pub fn store(&self) -> &Store {
        &self.store
    }

    pub fn is_running(&self) -> bool {
        self.running.is_some()
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.running.as_ref().map(|r| r.addr)
    }

    /// Construit l'AppState en réutilisant les Arcs partagés (dico/journal identiques).
    fn build_state(&self) -> AppState {
        AppState {
            config: Arc::new(self.config.clone()),
            backend: self.backend.clone(),
            dictionary: self.dictionary.clone(),
            store: self.store.clone(),
        }
    }

    /// Démarre le serveur (idempotent : renvoie l'adresse courante si déjà en marche).
    pub async fn start(&mut self) -> anyhow::Result<SocketAddr> {
        if let Some(r) = &self.running {
            return Ok(r.addr);
        }
        let app = build_app(self.build_state());
        let listener = TcpListener::bind((self.config.server.host.as_str(), self.config.server.port)).await?;
        let addr = listener.local_addr()?;
        let (shutdown, rx) = oneshot::channel::<()>();
        let handle = tokio::spawn(async move {
            let _ = axum::serve(listener, app)
                .with_graceful_shutdown(async move {
                    let _ = rx.await;
                })
                .await;
        });
        self.running = Some(RunningServer { addr, shutdown, handle });
        Ok(addr)
    }

    /// Arrête le serveur proprement (idempotent).
    pub async fn stop(&mut self) {
        if let Some(r) = self.running.take() {
            let _ = r.shutdown.send(());
            let _ = r.handle.await;
        }
    }

    /// Applique une nouvelle config et redémarre (host/port/backend). Reconstruit le
    /// backend ; conserve le MÊME dictionnaire et le MÊME journal (Arcs portés).
    pub async fn apply_config(&mut self, config: Config) -> anyhow::Result<SocketAddr> {
        self.stop().await;
        self.backend = backends::from_config(&config.backend);
        self.config = config;
        self.start().await
    }

    /// Recharge le dictionnaire à chaud (swap ArcSwap in-place, sans redémarrer).
    pub fn reload_dictionary(&self) -> std::io::Result<()> {
        self.dictionary.reload()
    }

    pub async fn backend_health(&self) -> BackendHealth {
        self.backend.health().await
    }
}

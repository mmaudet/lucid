//! Lucid — noyau headless : API compatible OpenAI + correction FR via Luciole-1B.

pub mod backends;
pub mod config;
pub mod correction;
pub mod dictionary;
pub mod openai;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_non_vide() {
        assert!(!version().is_empty());
    }
}

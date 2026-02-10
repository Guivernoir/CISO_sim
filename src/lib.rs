pub mod core;
pub mod narrative;
pub mod ui;

pub use core::*;
pub use narrative::*;
pub use ui::*;

use argon2::{Argon2, Params};
use argon2::password_hash::{PasswordHasher, SaltString};
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, AES_256_GCM};
use ring::error::Unspecified;
use std::fs;
use std::path::Path;

/// Encrypted save/load using AES-256-GCM for state persistence with Argon2 key derivation
pub struct GamePersistence {
    encryption_key: [u8; 32],
}

impl GamePersistence {
    /// Create a new persistence instance with proper key derivation
    pub fn new(password: &str) -> Result<Self> {
        let mut rng = rand::thread_rng();
        let salt = SaltString::generate(&mut rng);
        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            Params::new(150_000, 2, 1, Some(32)).map_err(|_| GameError::SystemFailure)?,
        );

        let key = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| GameError::SystemFailure)?
            .hash
            .ok_or(GameError::SystemFailure)?
            .as_bytes()
            .try_into()
            .map_err(|_| GameError::SystemFailure)?;

        Ok(Self { encryption_key: key })
    }

    pub fn save(&self, state: &GameState, path: &Path) -> Result<()> {
        let serialized = bincode::serialize(state).map_err(|_| GameError::StateCorruption)?;

        // Encrypt the state
        let encrypted = self.encrypt(&serialized)?;

        fs::write(path, encrypted).map_err(|_| GameError::SystemFailure)?;

        Ok(())
    }

    pub fn load(&self, path: &Path) -> Result<GameState> {
        let encrypted = fs::read(path).map_err(|_| GameError::SystemFailure)?;

        let decrypted = self.decrypt(&encrypted)?;

        let state = bincode::deserialize(&decrypted).map_err(|_| GameError::StateCorruption)?;

        Ok(state)
    }

    fn encrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.encryption_key)
            .map_err(|_| GameError::SystemFailure)?;

        let nonce_sequence = CounterNonceSequence::new();
        let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);

        let mut in_out = data.to_vec();
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut in_out)
            .map_err(|_| GameError::SystemFailure)?;

        Ok(in_out)
    }

    fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, &self.encryption_key)
            .map_err(|_| GameError::SystemFailure)?;

        let nonce_sequence = CounterNonceSequence::new();
        let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);

        let mut in_out = data.to_vec();
        let decrypted = opening_key
            .open_in_place(Aad::empty(), &mut in_out)
            .map_err(|_| GameError::StateCorruption)?;

        Ok(decrypted.to_vec())
    }
}

#[derive(Debug)]
struct CounterNonceSequence(u64);

impl CounterNonceSequence {
    fn new() -> Self {
        use rand::RngCore;
        let mut rng = rand::thread_rng();
        Self(rng.next_u64())
    }
}

impl NonceSequence for CounterNonceSequence {
    fn advance(&mut self) -> std::result::Result<Nonce, Unspecified> {
        let mut nonce_bytes = [0u8; 12];
        nonce_bytes[4..].copy_from_slice(&self.0.to_be_bytes());
        self.0 = self.0.wrapping_add(1);
        Nonce::try_assume_unique_for_key(&nonce_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_creation() {
        let state = GameState::new(
            Player::new(
                "Test Player".to_string(),
                "Test Company".to_string(),
                "Previous Role".to_string(),
            ),
        );
        assert_eq!(state.turn, 1);
        assert_eq!(state.quarter, 1);
        assert_eq!(state.phase, GamePhase::InheritanceDisaster);
    }

    #[test]
    fn test_risk_accumulation() {
        let mut risk = RiskLevel::new();
        let mut delta = RiskDelta::new();
        delta.add_change(RiskVector::DataExposure, 10.0, 0.0, 0.0);
        delta.add_change(RiskVector::AccessControl, 5.0, 0.0, 0.0);
        risk.apply_delta(&delta);
        assert_eq!(
            risk.vectors
                .get(&RiskVector::DataExposure)
                .unwrap()
                .current_level,
            10.0
        );
        assert_eq!(
            risk.vectors
                .get(&RiskVector::AccessControl)
                .unwrap()
                .current_level,
            5.0
        );
    }

    #[test]
    fn test_persistence_roundtrip() -> Result<()> {
        let persistence = GamePersistence::new("test_password")?;
        let original_state = GameState::new(
            Player::new(
                "Test".to_string(),
                "Company".to_string(),
                "Role".to_string(),
            ),
        );
        let path = Path::new("test_save.enc");
        persistence.save(&original_state, path)?;
        let loaded_state = persistence.load(path)?;
        assert_eq!(original_state.player.name, loaded_state.player.name);
        fs::remove_file(path).ok();
        Ok(())
    }
}
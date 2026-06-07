use crate::entropy;
use crate::vault;

/// Генерирует хеш намерения через Argon2id.
/// Пароль = текст намерения, соль = случайная энтропия.
/// Результат используется для XOR с основной энтропией.
pub fn hash_intent(intent_text: &str) -> [u8; 32] {
    let salt = entropy::generate_random_bytes(16);
    vault::get_master_key(intent_text, &salt, vault::SecurityProfile::Fast)
        .unwrap_or_else(|_| {
            // Fallback: SHAKE256 от текста намерения
            use tiny_keccak::{Hasher, Shake, Xof};
            let mut hasher = Shake::v256();
            hasher.update(intent_text.as_bytes());
            hasher.update(&salt);
            let mut out = [0u8; 32];
            hasher.squeeze(&mut out);
            out
        })
}

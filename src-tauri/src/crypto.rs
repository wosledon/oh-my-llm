use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use ring::rand::{SecureRandom, SystemRandom};
use std::sync::OnceLock;

static CRYPTO_KEY: OnceLock<Vec<u8>> = OnceLock::new();

fn get_or_init_key() -> &'static [u8] {
    CRYPTO_KEY.get_or_init(|| {
        let machine_id = format!(
            "{}-{}",
            std::env::consts::OS,
            option_env!("CARGO_PKG_NAME").unwrap_or("oh-my-llm")
        );
        use ring::digest::{digest, SHA256};
        let hash = digest(&SHA256, machine_id.as_bytes());
        hash.as_ref().to_vec()
    })
}

fn make_key_bytes() -> [u8; 32] {
    let key = get_or_init_key();
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&key[..32]);
    bytes
}

fn make_nonce() -> Result<[u8; NONCE_LEN], String> {
    let rng = SystemRandom::new();
    let mut nonce = [0u8; NONCE_LEN];
    rng.fill(&mut nonce)
        .map_err(|_| "Failed to generate nonce")?;
    Ok(nonce)
}

pub fn encrypt(plaintext: &str) -> Result<Vec<u8>, String> {
    let key_bytes = make_key_bytes();
    let nonce_bytes = make_nonce()?;

    let unbound_key =
        UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|_| "Failed to create key")?;
    let key = LessSafeKey::new(unbound_key);
    let nonce = Nonce::try_assume_unique_for_key(&nonce_bytes).map_err(|_| "Invalid nonce")?;

    let mut ciphertext = plaintext.as_bytes().to_vec();
    key.seal_in_place_append_tag(nonce, Aad::empty(), &mut ciphertext)
        .map_err(|_| "Encryption failed")?;

    let mut result = nonce_bytes.to_vec();
    result.extend_from_slice(&ciphertext);
    Ok(result)
}

pub fn decrypt(ciphertext: &[u8]) -> Result<String, String> {
    if ciphertext.len() < NONCE_LEN + 16 {
        return Err("Invalid ciphertext length".to_string());
    }

    let nonce_bytes = &ciphertext[..NONCE_LEN];
    let mut in_out = ciphertext[NONCE_LEN..].to_vec();

    let key_bytes = make_key_bytes();
    let unbound_key =
        UnboundKey::new(&AES_256_GCM, &key_bytes).map_err(|_| "Failed to create key")?;
    let key = LessSafeKey::new(unbound_key);
    let nonce = Nonce::try_assume_unique_for_key(nonce_bytes).map_err(|_| "Invalid nonce")?;

    let decrypted = key
        .open_in_place(nonce, Aad::empty(), &mut in_out)
        .map_err(|_| "Decryption failed")?;

    String::from_utf8(decrypted.to_vec()).map_err(|e| format!("Invalid UTF-8: {}", e))
}

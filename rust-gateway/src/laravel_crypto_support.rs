use aes::Aes256;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use cbc::{Decryptor, Encryptor};
use cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hmac::{Hmac, Mac};
use rand::{rngs::OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
struct LaravelEncryptedPayload {
    iv: String,
    value: String,
    mac: String,
}

#[derive(Serialize)]
struct LaravelEncryptedOutput<'a> {
    iv: &'a str,
    value: &'a str,
    mac: &'a str,
}

pub(crate) fn encrypt_laravel_string(app_key: &str, plaintext: &str) -> Result<String, String> {
    let key = decode_laravel_app_key(app_key)?;
    let mut iv = [0_u8; 16];
    OsRng.fill_bytes(&mut iv);

    let mut buffer = vec![0_u8; plaintext.as_bytes().len() + 16];
    buffer[..plaintext.as_bytes().len()].copy_from_slice(plaintext.as_bytes());
    let ciphertext = Encryptor::<Aes256>::new_from_slices(&key, &iv)
        .map_err(|err| format!("build laravel encryptor failed: {err}"))?
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext.as_bytes().len())
        .map_err(|err| format!("encrypt laravel value failed: {err}"))?;

    let iv = STANDARD.encode(iv);
    let value = STANDARD.encode(ciphertext);
    let mac = laravel_mac_hex(&key, &iv, &value)?;
    let envelope = LaravelEncryptedOutput {
        iv: &iv,
        value: &value,
        mac: &mac,
    };
    let payload = serde_json::to_vec(&envelope)
        .map_err(|err| format!("encode laravel payload failed: {err}"))?;
    Ok(STANDARD.encode(payload))
}

pub(crate) fn decrypt_laravel_string(app_key: &str, encrypted: &str) -> Result<String, String> {
    let key = decode_laravel_app_key(app_key)?;
    let envelope_bytes = STANDARD
        .decode(encrypted.trim())
        .map_err(|err| format!("decode laravel payload failed: {err}"))?;
    let envelope: LaravelEncryptedPayload = serde_json::from_slice(&envelope_bytes)
        .map_err(|err| format!("parse laravel payload failed: {err}"))?;

    verify_laravel_mac(&key, &envelope)?;

    let iv = STANDARD
        .decode(envelope.iv.trim())
        .map_err(|err| format!("decode laravel iv failed: {err}"))?;
    let mut ciphertext = STANDARD
        .decode(envelope.value.trim())
        .map_err(|err| format!("decode laravel value failed: {err}"))?;
    let plaintext = Decryptor::<Aes256>::new_from_slices(&key, &iv)
        .map_err(|err| format!("build laravel decryptor failed: {err}"))?
        .decrypt_padded_mut::<Pkcs7>(&mut ciphertext)
        .map_err(|err| format!("decrypt laravel value failed: {err}"))?;

    String::from_utf8(plaintext.to_vec()).map_err(|err| format!("laravel value is not utf8: {err}"))
}

fn decode_laravel_app_key(app_key: &str) -> Result<Vec<u8>, String> {
    let trimmed = app_key.trim();
    let key = if let Some(encoded) = trimmed.strip_prefix("base64:") {
        STANDARD
            .decode(encoded)
            .map_err(|err| format!("decode APP_KEY failed: {err}"))?
    } else {
        trimmed.as_bytes().to_vec()
    };
    if key.len() != 32 {
        return Err(format!("APP_KEY must decode to 32 bytes, got {}", key.len()));
    }
    Ok(key)
}

fn verify_laravel_mac(key: &[u8], envelope: &LaravelEncryptedPayload) -> Result<(), String> {
    let expected = laravel_mac_hex(key, &envelope.iv, &envelope.value)?;
    if !constant_time_eq(expected.as_bytes(), envelope.mac.as_bytes()) {
        return Err("laravel payload mac mismatch".to_string());
    }
    Ok(())
}

fn laravel_mac_hex(key: &[u8], iv: &str, value: &str) -> Result<String, String> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|err| format!("build laravel mac failed: {err}"))?;
    mac.update(format!("{iv}{value}").as_bytes());
    Ok(hex_lower(&mac.finalize().into_bytes()))
}

fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(HEX[(byte >> 4) as usize] as char);
        out.push(HEX[(byte & 0x0f) as usize] as char);
    }
    out
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut diff = 0_u8;
    for (left, right) in left.iter().zip(right.iter()) {
        diff |= left ^ right;
    }
    diff == 0
}

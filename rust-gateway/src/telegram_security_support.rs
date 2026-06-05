use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub(crate) fn telegram_webhook_secret_token(bot_token: &str, app_key: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(app_key.as_bytes())
        .expect("HMAC accepts keys of any length");
    mac.update(bot_token.as_bytes());
    hex_lower(mac.finalize().into_bytes().as_slice())
}

pub(crate) fn telegram_webhook_secret_matches(provided: &str, bot_token: &str, app_key: &str) -> bool {
    let provided = provided.trim();
    !provided.is_empty()
        && crate::secure_compare_support::constant_time_eq(
            provided.as_bytes(),
            telegram_webhook_secret_token(bot_token, app_key).as_bytes(),
        )
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

use sha2::{Digest, Sha256};
use std::fmt::Write as _;

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let digest_bytes: &[u8] = digest.as_ref();
    let mut output = String::with_capacity(digest_bytes.len() * 2);
    for byte in digest_bytes {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

#[cfg(test)]
mod tests {
    use super::sha256_hex;

    #[test]
    fn sha256_hex_returns_lowercase_digest() {
        assert_eq!(
            sha256_hex(b"kcu"),
            "9337aa21d30ea2127eb55c8c8ed448bccda8842a05879e86c0fa0f0e1c376e5a"
        );
    }
}

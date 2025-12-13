use anyhow::{anyhow, Result};
use totp_rs::{Algorithm, Secret, TOTP};
use url::Url;

pub fn generate_totp(otp_url: &str) -> Result<String> {
    let url = Url::parse(otp_url).map_err(|e| anyhow!("Failed to parse TOTP URL: {}", e))?;

    if url.scheme() != "otpauth" {
        return Err(anyhow!("Invalid scheme"));
    }

    if url.host_str() != Some("totp") {
        return Err(anyhow!("Only TOTP is supported"));
    }

    let secret_str = url
        .query_pairs()
        .find(|(k, _)| k == "secret")
        .map(|(_, v)| v.to_string())
        .ok_or_else(|| anyhow!("No secret found in URL"))?;

    // Remove spaces and padding
    let clean_secret = secret_str.replace(' ', "").replace('=', "");

    let secret = Secret::Encoded(clean_secret);
    let totp = TOTP::new(Algorithm::SHA1, 6, 1, 30, secret.to_bytes().unwrap())
        .map_err(|e| anyhow!("Failed to create TOTP instance: {}", e))?;
    totp.generate_current()
        .map_err(|e| anyhow!("Failed to generate TOTP: {}", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_totp_from_url() {
        // Example URL: otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXP&issuer=Example
        // Using longer secret: JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP (20 bytes)
        let url = "otpauth://totp/Example:alice@google.com?secret=JBSWY3DPEHPK3PXPJBSWY3DPEHPK3PXP&issuer=Example";
        let token = generate_totp(url);
        if let Err(e) = &token {
            println!("Error: {}", e);
        }
        assert!(token.is_ok());
        let token = token.unwrap();
        assert_eq!(token.len(), 6);
        assert!(token.chars().all(char::is_numeric));
    }
}

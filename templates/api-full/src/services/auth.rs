//! Authentication service

use crate::models::{TokenClaims, User, UserRole};
use armature::Provider;
use chrono::{Duration, Utc};
use std::any::Any;

pub struct AuthService {
    secret: String,
    expiry_hours: i64,
}

impl AuthService {
    pub fn new(secret: String) -> Self {
        Self {
            secret,
            expiry_hours: 24,
        }
    }

    pub fn with_expiry(mut self, hours: i64) -> Self {
        self.expiry_hours = hours;
        self
    }

    /// Hash a password (simple implementation - use bcrypt/argon2 in production)
    pub fn hash_password(&self, password: &str) -> String {
        // In production, use bcrypt or argon2
        // This is a placeholder for demonstration
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        password.hash(&mut hasher);
        self.secret.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Verify a password against a hash
    pub fn verify_password(&self, password: &str, hash: &str) -> bool {
        self.hash_password(password) == hash
    }

    /// Generate a JWT token for a user
    pub fn generate_token(&self, user: &User) -> Result<String, String> {
        let now = Utc::now();
        let exp = now + Duration::hours(self.expiry_hours);

        let claims = TokenClaims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            role: user.role,
            iat: now.timestamp() as usize,
            exp: exp.timestamp() as usize,
        };

        // In production, use the jsonwebtoken crate
        // This is a simple base64 encoding for demonstration
        let claims_json = serde_json::to_string(&claims).map_err(|e| e.to_string())?;
        let encoded = base64_encode(&claims_json);
        Ok(format!("Bearer.{}.signature", encoded))
    }

    /// Verify a JWT token and extract claims
    pub fn verify_token(&self, token: &str) -> Result<TokenClaims, String> {
        let token = token.trim_start_matches("Bearer ");

        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err("Invalid token format".to_string());
        }

        let claims_json = base64_decode(parts[1])?;
        let claims: TokenClaims =
            serde_json::from_str(&claims_json).map_err(|e| e.to_string())?;

        // Check expiration
        let now = Utc::now().timestamp() as usize;
        if claims.exp < now {
            return Err("Token expired".to_string());
        }

        Ok(claims)
    }

    /// Get token expiry in seconds
    pub fn token_expiry_seconds(&self) -> u64 {
        (self.expiry_hours * 3600) as u64
    }
}

impl Provider for AuthService {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn base64_encode(input: &str) -> String {
    use std::io::Write;
    let mut output = Vec::new();
    {
        let mut encoder = Base64Encoder::new(&mut output);
        encoder.write_all(input.as_bytes()).unwrap();
    }
    String::from_utf8(output).unwrap()
}

fn base64_decode(input: &str) -> Result<String, String> {
    let decoded = Base64Decoder::decode(input)?;
    String::from_utf8(decoded).map_err(|e| e.to_string())
}

// Simple base64 implementation (use base64 crate in production)
struct Base64Encoder<'a> {
    output: &'a mut Vec<u8>,
}

impl<'a> Base64Encoder<'a> {
    fn new(output: &'a mut Vec<u8>) -> Self {
        Self { output }
    }
}

impl<'a> std::io::Write for Base64Encoder<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

        for chunk in buf.chunks(3) {
            let b0 = chunk[0] as usize;
            let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
            let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

            self.output.push(ALPHABET[b0 >> 2]);
            self.output.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)]);

            if chunk.len() > 1 {
                self.output.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)]);
            } else {
                self.output.push(b'=');
            }

            if chunk.len() > 2 {
                self.output.push(ALPHABET[b2 & 0x3f]);
            } else {
                self.output.push(b'=');
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct Base64Decoder;

impl Base64Decoder {
    fn decode(input: &str) -> Result<Vec<u8>, String> {
        const DECODE_TABLE: [i8; 256] = {
            let mut table = [-1i8; 256];
            let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut i = 0;
            while i < 64 {
                table[alphabet[i] as usize] = i as i8;
                i += 1;
            }
            table
        };

        let input = input.trim_end_matches('=');
        let mut output = Vec::with_capacity(input.len() * 3 / 4);

        let bytes: Vec<u8> = input
            .bytes()
            .filter_map(|b| {
                let val = DECODE_TABLE[b as usize];
                if val >= 0 {
                    Some(val as u8)
                } else {
                    None
                }
            })
            .collect();

        for chunk in bytes.chunks(4) {
            if chunk.len() >= 2 {
                output.push((chunk[0] << 2) | (chunk[1] >> 4));
            }
            if chunk.len() >= 3 {
                output.push((chunk[1] << 4) | (chunk[2] >> 2));
            }
            if chunk.len() >= 4 {
                output.push((chunk[2] << 6) | chunk[3]);
            }
        }

        Ok(output)
    }
}


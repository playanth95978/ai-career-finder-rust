//! Known-sentinel JWT_SECRET values that the application refuses to start with.
//!
//! These literals are the historical defaults shipped by the generator templates
//! and bundled K8s/Helm manifests. If the application sees JWT_SECRET set to any
//! of these values, the operator hasn't replaced the placeholder — refusing to
//! start is safer than silently accepting a publicly-known signing key.
//!
//! This is the same denylist enforced by `docker-entrypoint.sh` at container
//! start. Defense-in-depth: the entrypoint catches `docker run` cases; this
//! Rust check catches the K8s static-manifest path where envFrom: secretRef
//! populates JWT_SECRET to the unmodified sentinel literal from app-secret.yml
//! before the binary even starts.
//!
//! See RELEASE_NOTES.md (0.9.8) for migration guidance.

/// Exact-match sentinels (legacy default values from older scaffolds).
const EXACT_SENTINELS: &[&str] = &[
    "change-me-in-production",
    "change-me-in-production-use-a-secure-random-string",
    "your-super-secret-jwt-key-change-in-production",
];

/// Pattern marker for the older timestamp-based default emitted by env.ejs
/// before 0.9.8: `<baseName>-jwt-secret-key-change-in-production-<timestamp>`.
/// Any value that contains this substring is rejected.
const PATTERN_MARKER: &str = "-jwt-secret-key-change-in-production-";

/// Returns true if the value matches a known-default sentinel and the application
/// must refuse to start. Empty values are also rejected: signing JWTs with a
/// zero-byte HMAC key produces forgable tokens.
pub fn is_sentinel(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    // Track 1-c.2 fix (2026-05-11): clippy::manual_contains flags iter().any()
    // when contains() is the idiomatic alternative on slices.
    if EXACT_SENTINELS.contains(&value) {
        return true;
    }
    if value.contains(PATTERN_MARKER) {
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_exact_sentinel_change_me() {
        assert!(is_sentinel("change-me-in-production-use-a-secure-random-string"));
    }

    #[test]
    fn rejects_bare_change_me_in_production() {
        // The shorter form historically shipped in K8s/Helm/Consul templates.
        // Without this entry, defense-in-depth would let it through.
        assert!(is_sentinel("change-me-in-production"));
    }

    #[test]
    fn rejects_empty_string() {
        // An empty JWT_SECRET signs every token with a zero-byte HMAC key —
        // trivially forgable. Refuse to start.
        assert!(is_sentinel(""));
    }

    #[test]
    fn rejects_exact_sentinel_your_super_secret() {
        assert!(is_sentinel("your-super-secret-jwt-key-change-in-production"));
    }

    #[test]
    fn rejects_legacy_timestamp_pattern() {
        // env.ejs prior to 0.9.8 emitted: <baseName>-jwt-secret-key-change-in-production-<ms>
        assert!(is_sentinel("myapp-jwt-secret-key-change-in-production-1777834673587"));
        assert!(is_sentinel("anything-jwt-secret-key-change-in-production-x"));
    }

    #[test]
    fn accepts_random_csprng_value() {
        // 0.9.8 default — 64-char hex from CSPRNG
        let csprng = "2ad022ee622493dde7ec5589c7416c15472d7e3dfc3399046ab2562035e9cfa7";
        assert!(!is_sentinel(csprng));
    }

    #[test]
    fn accepts_arbitrary_operator_value() {
        assert!(!is_sentinel("operator-supplied-real-secret"));
        assert!(!is_sentinel("a"));
    }
}

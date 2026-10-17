## Security Policy

### Supported Versions

Armature is a set of independently versioned crates. All of them are pre-1.0,
so every minor release (`0.x` → `0.(x+1)`) may contain breaking changes, and
security fixes land only on the **latest minor release line** of each crate.
Upgrade to the newest minor version to receive a fix; older lines are not
patched.

| Crate | Supported | Not supported |
| ----- | --------- | ------------- |
| `armature-framework` (umbrella) | `0.7.x` and newer | `< 0.7` |
| `armature-core` | `0.10.x` and newer | `< 0.10` |
| `armature-jwt` | `0.4.x` and newer | `< 0.4` |
| `armature-auth` | `0.4.x` and newer | `< 0.4` |
| `armature-security` | `0.4.x` and newer | `< 0.4` |
| `armature-h1` | `0.3.x` and newer | `< 0.3` |
| `armature-cli` | `0.7.x` and newer | `< 0.7` |
| every other `armature-*` crate | the latest published minor version on crates.io | any earlier minor version |

The project templates under `templates/` and the Compose stacks under
`docker/` are examples, not released artifacts: fixes are made on `main` and
are not backported. The Compose stacks are for local development only and must
not be exposed to a network.

### Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues,
discussions, or pull requests.**

Report privately through **GitHub private vulnerability reporting**:
[open a draft security advisory](https://github.com/quinnjr/armature/security/advisories/new).
For a vulnerability in a single crate you may also use that crate's own
repository (`https://github.com/quinnjr/armature-<name>/security/advisories/new`).
The report stays visible only to you and the maintainers until a fix is published.

The same contact is published in machine-readable form (RFC 9116) at
<https://quinnjr.github.io/armature/.well-known/security.txt>. The documentation
site is a GitHub Pages project site, so the file lives under `/armature/`
rather than at the host root, where some scanners look for it.

If you cannot use GitHub, email [quinn.josephr@proton.me](mailto:quinn.josephr@proton.me)
with `[armature security]` in the subject. You may encrypt with the maintainer's
OpenPGP key from <https://github.com/quinnjr.gpg>.

#### What to Include

Please include the following information in your report:

- Type of vulnerability
- Full paths of source file(s) related to the vulnerability
- Location of the affected source code (tag/branch/commit or direct URL)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the vulnerability, including how an attacker might exploit it

#### Response Timeline

- **Acknowledgment**: Within 48 hours
- **Initial Assessment**: Within 7 days
- **Fix Timeline**: Depends on severity
  - Critical: Within 7 days
  - High: Within 14 days
  - Medium: Within 30 days
  - Low: Next release cycle

#### Disclosure Policy

- Security vulnerabilities are disclosed after a fix is available
- We will credit reporters unless they wish to remain anonymous
- We follow a coordinated disclosure process
- Public disclosure occurs after users have had time to update (typically 2 weeks after release)

### Security Best Practices

When using Armature in production:

1. **Keep Dependencies Updated**
   - Run `cargo audit` regularly
   - Update dependencies promptly when security patches are released
   - Use Dependabot or similar tools for automated updates

2. **Authentication & Authorization**
   - Use strong JWT secrets: at least 32 random bytes for HS256 (e.g. `openssl rand -base64 48`); `JwtConfig::new` returns an error for shorter, placeholder or low-entropy secrets
   - Authenticate requests with `JwtManager::verify_access` (or `armature_auth::JwtAuthMiddleware`, which uses it); refresh tokens are not accepted as bearer tokens
   - Register authentication middleware with `Application::use_middleware` so it runs before guards; guards fail closed without a verified identity
   - Enable HTTPS/TLS for all deployments, and list your public hosts with `Application::with_https_redirect_hosts` when you use the HTTP→HTTPS redirect
   - Implement proper RBAC using guards
   - Never store sensitive data in JWT payload
   - Use short token expiration times

3. **Input Validation**
   - Use the validation framework for all user inputs
   - Sanitize data before database operations
   - Validate file uploads strictly
   - Implement rate limiting on sensitive endpoints
   - Set `trusted_proxy_depth` to the exact number of reverse proxies in front of the app; `X-Forwarded-For` is ignored at the default depth `0`
   - Leave SVG uploads disabled unless you sanitize them (`FileValidator::allow_svg_unsanitized()` is an explicit opt-in)

4. **Security Headers**
   - Use `armature-security` middleware for HTTP security headers
   - Configure Content Security Policy (CSP) appropriately
   - Enable HSTS for HTTPS deployments
   - Set appropriate CORS policies (`allow_origin_regex` patterns match the whole origin)
   - Use `armature_security::csrf::CsrfMiddleware` when you authenticate with cookies

5. **Secrets Management**
   - Never commit secrets to version control
   - Use environment variables or secret management services
   - Rotate secrets regularly
   - Use different secrets for development/staging/production

6. **Database Security**
   - Use parameterized queries (Armature does this by default)
   - Apply principle of least privilege for database accounts
   - Encrypt sensitive data at rest
   - Enable database audit logging

7. **Monitoring & Logging**
   - Enable OpenTelemetry observability
   - Monitor for unusual patterns
   - Log security-relevant events
   - Set up alerts for suspicious activity
   - Do not log sensitive data (passwords, tokens, etc.)

8. **Deployment**
   - Run containers as non-root user
   - Use minimal base images
   - Scan container images for vulnerabilities
   - Keep host OS and runtime updated
   - Enable firewall rules

### Security Features

Armature provides several built-in security features:

- **JWT Authentication** (`armature-jwt`)
  - Industry-standard token-based auth
  - HMAC, RSA (RS/PS), ECDSA and EdDSA algorithms
  - Rejects weak HMAC secrets; access and refresh tokens are distinguished by claim and `typ` header

- **OAuth2/OIDC** (`armature-auth`)
  - Provider integrations (Google, Microsoft, AWS Cognito, Okta, Auth0)
  - PKCE (S256), library-verified `state` and OIDC `nonce` on every flow
  - `id_token` verification against the provider's JWKS

- **SAML 2.0** (`armature-auth`)
  - Enterprise SSO support
  - Service Provider implementation
  - Signature verification and assertion replay protection

- **Security Middleware** (`armature-security`)
  - Helmet-like security headers
  - CSP, HSTS, X-Frame-Options, COOP, CORP and Permissions-Policy
  - Signed double-submit CSRF protection (`csrf` module)
  - HMAC request signing

- **HTTPS/TLS** (`armature-core`)
  - Built-in TLS support
  - Certificate management
  - HTTP to HTTPS redirect to an allowlist of hosts
  - Header-read, TLS-handshake and request-body deadlines, plus a connection cap

- **Rate Limiting** (`armature-ratelimit`)
  - Multiple algorithms (Token Bucket, Sliding Window, etc.)
  - Configurable limits per endpoint
  - Protection against abuse

- **Input Validation** (`armature-validation`)
  - 18+ built-in validators
  - Custom rule builders
  - Automatic request validation

### Known Security Limitations

- **No built-in XSS protection**: Sanitize user input and output in your application
- **No automatic SQL injection protection**: Use proper ORM/query builders
- **Session management**: Armature is stateless by default. Server-side sessions are available in `armature-session`; call `SessionStore::regenerate` after login or any privilege change to prevent session fixation

### Security Audit

Armature has not yet undergone a professional security audit. We welcome:

- Security researchers to review our code
- Penetration testing reports
- Recommendations for security improvements

### Dependencies

We actively monitor our dependencies for security vulnerabilities using:

- `cargo-audit` in CI/CD pipeline (blocking) and in the pre-push hook
- GitHub Dependabot
- Secret scanning with gitleaks (configuration in `.gitleaks.toml`)

### Contact

For security-related questions or concerns, contact:

- **Security Reports**: Via [GitHub Security Advisories](https://github.com/quinnjr/armature/security/advisories/new) (email [quinn.josephr@proton.me](mailto:quinn.josephr@proton.me) only if you cannot use GitHub)
- **General Questions**: Open a [GitHub Issue](https://github.com/quinnjr/armature/issues)
- **Maintainer**: Joseph Quinn

### Attribution

We would like to thank the following researchers for responsibly disclosing security issues:

<!-- Will be populated as issues are reported and fixed -->

---

**Note**: This security policy is subject to change. Check back regularly for updates.

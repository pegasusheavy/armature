# Documentation Test Coverage Status

Current status and roadmap for documentation test coverage across all workspace members.

## Summary

**Current Status:** MEDIUM Coverage
**Total Doc Tests:** 60
**Average per Module:** 2.7
**Target:** 100+ tests (5+ per module average)

## Coverage by Module

### ✅ GOOD Coverage (5+ tests)

| Module | Tests | Status |
|--------|-------|--------|
| armature-cache | 14 | ✓ Excellent |
| armature-acme | 15 | ✓ Excellent |
| armature-graphql | 6 | ✓ Good |
| armature-jwt | 5 | ✓ Good |

**Total: 40 tests across 4 modules**

### ⚠️ LOW Coverage (1-4 tests)

| Module | Tests | Priority | Notes |
|--------|-------|----------|-------|
| armature-xss | 4 | Medium | Add sanitizer examples |
| armature-core | 3 | HIGH | Core module needs more examples |
| armature-config | 3 | Low | Basic coverage adequate |
| armature-validation | 3 | Medium | Add validator examples |
| armature-cron | 3 | Low | Add schedule examples |
| armature-queue | 2 | Medium | Add job processing examples |
| armature-csrf | 2 | Medium | Add middleware examples |

**Total: 20 tests across 7 modules**

### ❌ NO TESTS (0 tests)

| Module | Priority | Reason / Action |
|--------|----------|-----------------|
| armature-testing | HIGH | Test utilities need examples |
| armature-auth | HIGH | OAuth2/SAML examples needed |
| armature-security | HIGH | Helmet-like headers examples |
| armature-openapi | MEDIUM | Swagger generation examples |
| armature-handlebars | MEDIUM | Template rendering examples |
| armature-opentelemetry | LOW | Tracing setup examples |
| armature-angular | LOW | SSR examples (complex) |
| armature-react | LOW | SSR examples (complex) |
| armature-vue | LOW | SSR examples (complex) |
| armature-svelte | LOW | SSR examples (complex) |
| armature-macro | N/A | Procedural macros (hard to test) |

**Total: 0 tests across 11 modules**

## Roadmap

### Phase 1: Critical Modules (Target: +25 tests)

**Priority HIGH modules:**

1. **armature-testing** (+5 tests)
   - TestApp builder example
   - TestClient HTTP examples
   - MockService examples
   - Spy examples
   - Assertion examples

2. **armature-auth** (+5 tests)
   - OAuth2 flow example
   - SAML authentication example
   - JWT middleware example
   - Role guard example
   - Session management example

3. **armature-security** (+5 tests)
   - CSP header example
   - HSTS example
   - X-Frame-Options example
   - Complete security setup example
   - Per-route security example

4. **armature-core** (+5 tests)
   - HttpRequest examples
   - HttpResponse builder examples
   - Routing examples
   - Middleware chain example
   - Application setup example

5. **armature-validation** (+5 tests)
   - Email validator example
   - Custom validator example
   - Rules builder example
   - Async validation example
   - Error handling example

### Phase 2: Medium Priority (+15 tests)

1. **armature-openapi** (+3 tests)
2. **armature-handlebars** (+3 tests)
3. **armature-queue** (+3 tests)
4. **armature-cron** (+2 tests)
5. **armature-csrf** (+2 tests)
6. **armature-xss** (+2 tests)

### Phase 3: SSR Modules (+8 tests)

1. **armature-angular** (+2 tests)
2. **armature-react** (+2 tests)
3. **armature-vue** (+2 tests)
4. **armature-svelte** (+2 tests)

### Phase 4: Low Priority (+5 tests)

1. **armature-opentelemetry** (+3 tests)
2. **armature-config** (+2 tests)

## Testing Standards

### Every Doc Test Should:

✅ Compile without errors
✅ Demonstrate real-world usage
✅ Include necessary imports
✅ Handle errors appropriately
✅ Be concise and focused
✅ Use `no_run` for expensive operations
✅ Use `ignore` only when necessary

### Example Template:

```rust
/// Brief description of what this does.
///
/// # Examples
///
/// ```
/// use armature_module::{Type1, Type2};
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let instance = Type1::new("config");
/// let result = instance.method()?;
/// assert_eq!(result, expected);
/// # Ok(())
/// # }
/// ```
///
/// # Errors
///
/// Returns error when...
pub fn method() -> Result<T, Error> {
    // Implementation
}
```

## Progress Tracking

- [x] Infrastructure setup (test-docs.sh, CI workflow)
- [x] Coverage analysis tool (check-doc-coverage.sh)
- [x] Documentation guide (DOCUMENTATION_TESTING.md)
- [ ] Phase 1: Critical modules (+25 tests)
- [ ] Phase 2: Medium priority (+15 tests)
- [ ] Phase 3: SSR modules (+8 tests)
- [ ] Phase 4: Low priority (+5 tests)

**Target:** 113+ total doc tests

## Running Tests

```bash
# Check coverage
./scripts/check-doc-coverage.sh

# Run all doc tests
./scripts/test-docs.sh

# Run specific module
cargo test --doc -p armature-testing
```

## Continuous Improvement

- Review coverage monthly
- Add examples for new features immediately
- Keep examples up-to-date with API changes
- Aim for 5+ tests per module minimum
- Target 100+ tests total across workspace

## Notes

**Proc Macro Testing:** `armature-macro` is excluded because procedural macros are tested via integration tests in dependent crates, not via doc tests.

**SSR Modules:** Angular, React, Vue, Svelte have lower priority as they require complex setup (Node.js, build tools). Focus on core framework features first.

Last Updated: 2025-12-06


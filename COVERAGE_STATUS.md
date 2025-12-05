# Armature Test Coverage Status & Roadmap

## Current Status

**Date**: December 5, 2024
**Goal**: 85% test coverage across all modules
**Status**: In Progress

## Quick Summary

This document tracks the test coverage status of all Armature modules and provides a roadmap to achieve the 85% coverage target.

## Module Status

### Core Modules

#### armature-core ⚠️
**Status**: Needs significant test additions
**Existing Tests**: 18 tests
**Coverage Estimate**: ~45%

**Missing Coverage**:
- [ ] TLS module (tls.rs) - 0% coverage
- [ ] HTTP request/response handling edge cases
- [ ] Router path matching edge cases
- [ ] Error conversion paths
- [ ] Middleware chain execution
- [ ] Guard execution flows
- [ ] Interceptor execution flows

**Priority Actions**:
1. Add TLS certificate loading tests
2. Add TLS handshake simulation tests
3. Add HTTP redirect tests
4. Add router edge case tests (trailing slashes, query params, etc.)
5. Add error handling tests for all error variants

#### armature-macro ⚠️
**Status**: Difficult to test (procedural macros)
**Existing Tests**: Integration tests via examples
**Coverage Estimate**: ~60% (indirect)

**Approach**:
- Test macro output via integration tests
- Test generated code functionality
- Compile-time error handling

### Authentication & Authorization

#### armature-auth ✅
**Status**: Good coverage
**Existing Tests**: 19 tests
**Coverage Estimate**: ~75%

**Missing Coverage**:
- [ ] SAML response parsing edge cases
- [ ] OAuth2 state validation
- [ ] Token refresh flows
- [ ] Provider-specific error handling

#### armature-jwt ❌
**Status**: Tests failing, needs fixes
**Existing Tests**: 14 tests (3 failing)
**Coverage Estimate**: ~50%

**Priority Actions**:
1. Fix failing tests (missing `exp` claims)
2. Add expiration validation tests
3. Add key rotation tests
4. Add algorithm-specific tests (HS256, RS256, ES256)

### Frontend SSR Modules

#### armature-angular ✅
**Status**: Tests passing after fix
**Existing Tests**: 7 tests
**Coverage Estimate**: ~55%

**Missing Coverage**:
- [ ] Actual SSR rendering (requires Node.js setup)
- [ ] Static file serving edge cases
- [ ] Error handling for missing files

#### armature-react ⚠️
**Status**: Similar to angular
**Existing Tests**: Limited
**Coverage Estimate**: ~40%

#### armature-vue ⚠️
**Status**: Similar to angular
**Existing Tests**: Limited
**Coverage Estimate**: ~40%

#### armature-svelte ⚠️
**Status**: Similar to angular
**Existing Tests**: Limited
**Coverage Estimate**: ~40%

### API & Data

#### armature-graphql ✅
**Status**: Good coverage
**Existing Tests**: 4 tests
**Coverage Estimate**: ~65%

**Missing Coverage**:
- [ ] Schema merging
- [ ] Subscription handling
- [ ] Error propagation
- [ ] Context injection

#### armature-validation ✅
**Status**: Good coverage
**Existing Tests**: 8 tests
**Coverage Estimate**: ~70%

**Missing Coverage**:
- [ ] Async validation
- [ ] Complex validation rules
- [ ] Error message customization

#### armature-openapi ⚠️
**Status**: Needs comprehensive tests
**Existing Tests**: 1 test
**Coverage Estimate**: ~30%

**Priority Actions**:
1. Add spec building tests
2. Add Swagger UI integration tests
3. Add schema generation tests
4. Add parameter/response tests

### Infrastructure

#### armature-config ✅
**Status**: Good coverage
**Existing Tests**: 9 tests
**Coverage Estimate**: ~70%

**Missing Coverage**:
- [ ] TOML parsing edge cases
- [ ] Environment variable override conflicts
- [ ] Nested configuration validation

#### armature-cache ⚠️
**Status**: Needs tests
**Existing Tests**: 0 tests
**Coverage Estimate**: ~0%

**Priority Actions** (HIGH):
1. Add Redis connection tests
2. Add Memcached connection tests
3. Add TTL expiration tests
4. Add atomic operation tests
5. Add cache key tests

#### armature-cron ⚠️
**Status**: Needs tests
**Existing Tests**: 0 tests
**Coverage Estimate**: ~0%

**Priority Actions** (HIGH):
1. Add cron expression parsing tests
2. Add job scheduling tests
3. Add job execution tests
4. Add error handling tests

#### armature-queue ⚠️
**Status**: Needs tests
**Existing Tests**: 0 tests
**Coverage Estimate**: ~0%

**Priority Actions** (HIGH):
1. Add job enqueueing tests
2. Add worker processing tests
3. Add retry logic tests
4. Add priority queue tests

#### armature-opentelemetry ⚠️
**Status**: Minimal coverage
**Existing Tests**: 1 test
**Coverage Estimate**: ~20%

**Missing Coverage**:
- [ ] Tracer initialization
- [ ] Metrics collection
- [ ] Middleware instrumentation
- [ ] Exporter configuration

### Testing & Utilities

#### armature-testing ✅
**Status**: Self-testing utilities
**Existing Tests**: Tests via usage
**Coverage Estimate**: ~60%

**Approach**:
- Test through integration test usage
- Add specific utility tests

## Coverage Measurement

### Tools

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --all-features --out Html --output-dir coverage

# View report
open coverage/index.html
```

### Current Results

**Overall Estimate**: ~45-50% coverage

**High Coverage** (70%+):
- armature-auth
- armature-validation
- armature-config

**Medium Coverage** (40-70%):
- armature-core
- armature-graphql
- armature-angular
- armature-testing

**Low Coverage** (0-40%):
- armature-cache (0%)
- armature-cron (0%)
- armature-queue (0%)
- armature-opentelemetry (20%)
- armature-openapi (30%)
- armature-react (40%)
- armature-vue (40%)
- armature-svelte (40%)

## Roadmap to 85%

### Phase 1: Fix Failing Tests (Priority: CRITICAL)

**Tasks**:
1. ✅ Fix armature-angular tests
2. ❌ Fix armature-jwt tests (missing exp claims)
3. ❌ Verify all existing tests pass

**Time Estimate**: 2-4 hours

### Phase 2: Add Tests to Zero-Coverage Modules (Priority: HIGH)

**armature-cache**:
- Redis connection tests
- Memcached connection tests
- TTL and expiration tests
- Get/Set/Delete operations
- Atomic operations (incr/decr)

**armature-cron**:
- Cron expression parsing tests
- Job scheduling tests
- Job execution tests
- Statistics tracking tests

**armature-queue**:
- Job enqueueing tests
- Worker processing tests
- Retry logic tests
- Priority handling tests

**Time Estimate**: 8-12 hours

### Phase 3: Improve Core Module Coverage (Priority: HIGH)

**armature-core**:
- TLS certificate loading (from files, memory, self-signed)
- TLS configuration tests
- HTTP redirect tests
- Router edge cases
- Middleware chain tests
- Guard and interceptor tests

**Time Estimate**: 6-10 hours

### Phase 4: Enhance Existing Module Coverage (Priority: MEDIUM)

**armature-openapi**:
- Spec building tests
- Swagger UI tests
- Schema generation tests

**armature-opentelemetry**:
- Tracer initialization tests
- Metrics collection tests
- Middleware instrumentation tests

**armature-graphql**:
- Schema merging tests
- Subscription tests
- Context injection tests

**Time Estimate**: 6-8 hours

### Phase 5: SSR Module Integration Tests (Priority: LOW)

**armature-react/vue/svelte**:
- Renderer initialization tests
- Props serialization tests
- Error handling tests

**Note**: Full SSR testing requires Node.js setup and is complex

**Time Estimate**: 4-6 hours

## Testing Best Practices

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    // Helper functions
    fn setup() -> TestContext {
        // Setup code
    }

    #[test]
    fn test_feature_success_case() {
        // Arrange
        let context = setup();

        // Act
        let result = context.do_something();

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_feature_error_case() {
        // Arrange
        let context = setup();

        // Act
        let result = context.do_something_invalid();

        // Assert
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_async_feature() {
        let result = async_operation().await;
        assert!(result.is_ok());
    }
}
```

### Coverage Targets by Priority

**Critical Paths (100%)**:
- Error handling
- Security functions (auth, encryption)
- Data integrity operations

**Public APIs (90%+)**:
- All public functions
- All public methods
- All exported types

**Internal Implementation (70%+)**:
- Private functions
- Helper utilities
- Internal logic

**Edge Cases (Target as needed)**:
- Boundary conditions
- Error paths
- Unusual inputs

## Quick Wins

To quickly improve coverage:

1. **Add armature-cache tests** (~200 lines) → +10% coverage
2. **Add armature-cron tests** (~150 lines) → +8% coverage
3. **Add armature-queue tests** (~150 lines) → +8% coverage
4. **Add armature-core TLS tests** (~200 lines) → +5% coverage
5. **Fix armature-jwt tests** (~50 lines) → +2% coverage

**Total Quick Wins**: +33% coverage gain

## Commands Reference

```bash
# Run all tests
cargo test --workspace

# Run specific package tests
cargo test --package armature-core

# Run with coverage
cargo tarpaulin --workspace --all-features --out Html

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Check coverage percentage
cargo tarpaulin --workspace --all-features | grep "^Coverage"
```

## Current Blockers

1. ❌ JWT tests failing - need to fix exp claims
2. ⚠️  Cache/Cron/Queue modules have zero tests
3. ⚠️  TLS module needs comprehensive tests
4. ⚠️  SSR modules need Node.js integration testing

## Next Steps

**Immediate**:
1. Fix armature-jwt failing tests
2. Add tests to armature-cache
3. Add tests to armature-cron  
4. Add tests to armature-queue

**Short Term**:
1. Add TLS tests to armature-core
2. Enhance armature-openapi tests
3. Enhance armature-opentelemetry tests

**Long Term**:
1. Add comprehensive SSR integration tests
2. Add performance benchmark tests
3. Add end-to-end integration tests
4. Set up CI coverage reporting

## Resources

- [Testing Coverage Guide](./docs/TESTING_COVERAGE.md)
- [cargo-tarpaulin Documentation](https://github.com/xd009642/tarpaulin)
- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)

## Conclusion

Achieving 85% coverage is feasible with focused effort on the zero-coverage modules (cache, cron, queue) and enhancing core module tests. The estimated total time to reach 85% coverage is approximately **25-40 hours** of focused testing work.

**Priority order**:
1. Fix failing tests (Critical)
2. Add tests to zero-coverage modules (High - biggest impact)
3. Enhance core module coverage (High)
4. Improve existing module coverage (Medium)
5. Add SSR integration tests (Low - complex, less impact)

**Recommended Approach**:
Start with "Quick Wins" to rapidly improve coverage from ~50% to ~80%, then focus on remaining gaps to reach 85%+.


#!/bin/bash
# Create GitHub issues for high-priority performance TODOs
# Run: gh auth login (if not already authenticated)
# Then: ./scripts/create-issues.sh

set -e

REPO="pegasusheavy/armature"

echo "Creating GitHub issues for high-priority TODOs..."
echo "Repository: $REPO"
echo ""

# =============================================================================
# CRITICAL (🔴) - Axum Competitive
# =============================================================================

echo "Creating Critical Priority issues..."

gh issue create --repo "$REPO" \
  --title "perf: Replace Trie router with \`matchit\` crate" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Replace the current trie-based router with the \`matchit\` crate (same router used by Axum) for significantly faster route matching.

## Motivation
Based on CPU profiling, route matching accounts for ~7% of CPU time. The \`matchit\` crate is highly optimized and battle-tested in Axum.

## Implementation
- [ ] Add \`matchit\` dependency
- [ ] Refactor \`armature-core/routing.rs\` to use \`matchit::Router\`
- [ ] Update route registration to use matchit's API
- [ ] Benchmark before/after to quantify improvement

## Module
\`armature-core/routing.rs\`

## Priority
🔴 Critical - Required for Axum-competitive performance"

gh issue create --repo "$REPO" \
  --title "perf: Compile-time route validation" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Validate routes at compile time instead of runtime to catch errors early and eliminate runtime validation overhead.

## Module
\`armature-macro\`

## Priority
🔴 Critical"

gh issue create --repo "$REPO" \
  --title "perf: Inline handler dispatch via monomorphization" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Ensure handlers are inlined via monomorphization to eliminate virtual dispatch overhead.

## Implementation
- [ ] Audit handler dispatch path for dynamic dispatch
- [ ] Use generics instead of trait objects where possible
- [ ] Add \`#[inline]\` hints on critical path functions

## Module
\`armature-core\`

## Priority
🔴 Critical"

gh issue create --repo "$REPO" \
  --title "perf: Remove runtime type checks from DI hot paths" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Eliminate \`Any\` downcasting and runtime type checks in dependency injection hot paths.

## Module
\`armature-core/di.rs\`

## Priority
🔴 Critical"

gh issue create --repo "$REPO" \
  --title "perf: Per-request arena allocator" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Implement a per-request arena allocator to batch deallocations and reduce allocator pressure.

## Implementation
- [ ] Evaluate \`bumpalo\` or similar arena crates
- [ ] Create per-request arena context
- [ ] Migrate request-scoped allocations to arena

## Module
\`armature-core\`

## Priority
🔴 Critical"

gh issue create --repo "$REPO" \
  --title "perf: Direct Hyper body passthrough" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Avoid wrapping/unwrapping \`hyper::Body\` to reduce overhead in request/response handling.

## Module
\`armature-core\`

## Priority
🔴 Critical"

gh issue create --repo "$REPO" \
  --title "perf: TechEmpower Benchmark Suite" \
  --label "enhancement,performance,priority: critical,benchmarks" \
  --body "## Summary
Implement all TechEmpower benchmark tests (JSON, DB, Fortune, Plaintext) to measure framework performance.

## Implementation
- [ ] JSON serialization test
- [ ] Single database query
- [ ] Multiple database queries
- [ ] Fortune template rendering
- [ ] Plaintext response
- [ ] Data updates

## Module
\`benches/techempower/\`

## Priority
🔴 Critical"

# =============================================================================
# CRITICAL (🔴) - Actix Competitive
# =============================================================================

gh issue create --repo "$REPO" \
  --title "perf: HTTP/1.1 request pipelining" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Process multiple HTTP/1.1 requests per connection without waiting for responses (pipelining).

## Motivation
Actix-web excels at HTTP/1.1 pipelining, processing multiple requests from the socket buffer efficiently.

## Module
\`armature-core/http.rs\`

## Priority
🔴 Critical - Actix competitive"

gh issue create --repo "$REPO" \
  --title "perf: Request batching from socket buffer" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Batch-read multiple requests from socket buffer to reduce syscall overhead.

## Module
\`armature-core/http.rs\`

## Priority
🔴 Critical - Actix competitive"

gh issue create --repo "$REPO" \
  --title "perf: BytesMut buffer pool" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Implement thread-local pool of pre-allocated \`BytesMut\` buffers to reduce allocation overhead.

## Motivation
Buffer pooling is one of Actix-web's key performance advantages.

## Module
\`armature-core/buffer.rs\`

## Priority
🔴 Critical - Actix competitive"

gh issue create --repo "$REPO" \
  --title "perf: Zero-copy request body parsing" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Parse request bodies directly into pooled buffers without intermediate copying.

## Module
\`armature-core/request.rs\`

## Priority
🔴 Critical - Actix competitive"

gh issue create --repo "$REPO" \
  --title "perf: io_uring support for Linux" \
  --label "enhancement,performance,priority: critical" \
  --body "## Summary
Use io_uring for async I/O on Linux 5.1+ for significantly reduced syscall overhead.

## Implementation
- [ ] Add \`tokio-uring\` or \`glommio\` integration
- [ ] Feature-flag for io_uring vs epoll
- [ ] Benchmark syscall reduction

## Module
\`armature-core/io.rs\`

## Priority
🔴 Critical - Actix competitive"

gh issue create --repo "$REPO" \
  --title "perf: Actix-web comparison benchmark" \
  --label "enhancement,performance,priority: critical,benchmarks" \
  --body "## Summary
Create direct A/B benchmark comparing Armature vs Actix-web with identical routes.

## Module
\`benches/comparison/actix/\`

## Priority
🔴 Critical"

# =============================================================================
# HIGH PRIORITY (🟠) - Performance
# =============================================================================

echo "Creating High Priority issues..."

gh issue create --repo "$REPO" \
  --title "perf: Route matching cache" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Cache compiled routes to avoid repeated trie traversal on every request.

## Module
\`armature-core/routing.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Static route fast path" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Bypass trie for exact-match static routes using HashMap for O(1) lookup.

## Module
\`armature-core/routing.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: SIMD JSON serialization" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Add optional \`simd-json\` or \`sonic-rs\` for faster JSON serialization/deserialization.

## Module
\`armature-core\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Zero-allocation route parameter extraction" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Extract route parameters without allocation, similar to Axum's approach.

## Module
\`armature-core/routing.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Const generic extractors" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Use const generics for zero-cost extractor chains.

## Module
\`armature-core/extractors.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Static dispatch middleware" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Replace \`Box<dyn>\` with static dispatch where possible in middleware.

## Module
\`armature-core/middleware.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: SmallVec for headers" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Use \`SmallVec<[_; 16]>\` for typical header counts to avoid heap allocation.

## Module
\`armature-core\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Tower Service compatibility" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Implement \`tower::Service\` for middleware composability and ecosystem integration.

## Module
\`armature-core\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Reduce task spawning overhead" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Inline simple handlers instead of spawning tasks for each request.

## Module
\`armature-core\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Profile-Guided Optimization (PGO) build profile" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Add PGO build profile for optimized production builds.

## Module
\`Cargo.toml\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Vectored I/O (writev) for responses" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Use \`writev()\` to send headers+body in single syscall.

## Module
\`armature-core/http.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: CPU core affinity for workers" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Pin worker threads to CPU cores for better cache locality.

## Module
\`armature-core/runtime.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Connection recycling" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Reset and reuse connection objects instead of allocating new ones.

## Module
\`armature-core/connection.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Streaming response body support" \
  --label "enhancement,performance,priority: high" \
  --body "## Summary
Send response while still generating body for lower latency.

## Module
\`armature-core/response.rs\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "ci: Automated performance regression tests" \
  --label "enhancement,ci,priority: high" \
  --body "## Summary
CI pipeline to catch performance regressions on PRs.

## Module
\`.github/workflows/\`

## Priority
🟠 High"

gh issue create --repo "$REPO" \
  --title "perf: Axum comparison benchmark" \
  --label "enhancement,performance,priority: high,benchmarks" \
  --body "## Summary
Side-by-side benchmark vs Axum with identical routes.

## Module
\`benches/comparison/\`

## Priority
🟠 High"

# =============================================================================
# HIGH PRIORITY (🟠) - Enterprise
# =============================================================================

gh issue create --repo "$REPO" \
  --title "feat: i18n support - message translation" \
  --label "enhancement,priority: high" \
  --body "## Summary
Add internationalization support for message translation.

## Module
\`armature-i18n\`

## Priority
🟠 High - Enterprise"

gh issue create --repo "$REPO" \
  --title "feat: Locale detection from Accept-Language" \
  --label "enhancement,priority: high" \
  --body "## Summary
Parse Accept-Language header for automatic locale detection.

## Module
\`armature-i18n\`

## Priority
🟠 High - Enterprise"

echo ""
echo "✅ Done! Created all high-priority issues."
echo ""
echo "View issues at: https://github.com/$REPO/issues"


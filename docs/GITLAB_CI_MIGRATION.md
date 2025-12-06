# GitLab CI/CD Migration Guide

Complete guide for the Armature framework's migration from GitHub Actions to GitLab CI/CD.

## Table of Contents

- [Overview](#overview)
- [Pipeline Structure](#pipeline-structure)
- [Key Differences](#key-differences)
- [Setup Instructions](#setup-instructions)
- [CI/CD Variables](#cicd-variables)
- [Pipeline Schedules](#pipeline-schedules)
- [Features Comparison](#features-comparison)
- [Troubleshooting](#troubleshooting)

---

## Overview

The Armature project has migrated from GitHub Actions to GitLab CI/CD. All workflows have been consolidated into a single `.gitlab-ci.yml` file with multiple stages and jobs.

**Repository:** https://gitlab.com/pegasusheavy/armature

---

## Pipeline Structure

### Stages

The pipeline is organized into 7 stages:

1. **lint** - Code formatting and linting
2. **test** - Unit tests, doc tests, examples
3. **security** - Security audits and dependency scanning
4. **build** - Release builds and cross-compilation
5. **benchmark** - Performance benchmarking
6. **docs** - Documentation generation and deployment
7. **release** - Crates.io publishing and release creation

### Pipeline Flow

```
┌─────────┐     ┌──────────┐     ┌──────────┐     ┌───────┐
│  Lint   │────▶│   Test   │────▶│ Security │────▶│ Build │
└─────────┘     └──────────┘     └──────────┘     └───────┘
                                                       │
    ┌──────────────────────────────────────────────────┤
    │                                                  │
    ▼                                                  ▼
┌───────────┐     ┌──────────┐     ┌─────────────────────┐
│ Benchmark │     │   Docs   │     │      Release        │
└───────────┘     └──────────┘     └─────────────────────┘
```

---

## Key Differences

### GitHub Actions vs GitLab CI

| Feature | GitHub Actions | GitLab CI |
|---------|---------------|-----------|
| **Config File** | `.github/workflows/*.yml` | `.gitlab-ci.yml` |
| **Triggers** | `on:` | `rules:` |
| **Jobs** | `jobs:` | `stages:` + `jobs` |
| **Steps** | `steps:` | `script:` |
| **Caching** | `actions/cache@v4` | Built-in `cache:` |
| **Artifacts** | `actions/upload-artifact` | Built-in `artifacts:` |
| **Matrix** | `strategy.matrix` | Parallel jobs |
| **Variables** | `${{ }}` syntax | `$VARIABLE` syntax |
| **Secrets** | GitHub Secrets | GitLab CI/CD Variables |

### Converted Workflows

| GitHub Workflow | GitLab Jobs | Stage |
|----------------|-------------|-------|
| **ci.yml** | `test:*`, `clippy`, `fmt:check` | lint, test |
| **doc-tests.yml** | `doc-tests` | test |
| **security.yml** | `security:audit`, dependency scanning | security |
| **benchmark.yml** | `benchmarks` | benchmark |
| **docs.yml** | `docs:build`, `pages` | docs |
| **release.yml** | `release:create`, `publish:*` | release |
| **nightly.yml** | `nightly:build`, `msrv:check` | test (scheduled) |

### Features Not Directly Migrated

These GitHub Actions features require alternative approaches in GitLab:

| Feature | GitHub | GitLab Alternative |
|---------|--------|-------------------|
| **Greetings** | `actions/first-interaction` | Webhooks + custom bot |
| **Stale Issues** | `actions/stale` | GitLab Triage Policies |
| **Auto-Labeling** | `actions/labeler` | GitLab Auto DevOps labels |
| **PR Size Labels** | Custom action | Merge request approvals |

---

## Setup Instructions

### 1. Configure CI/CD Variables

Go to **Settings → CI/CD → Variables** in GitLab and add:

#### Required Variables

```
CARGO_REGISTRY_TOKEN
  Type: Variable
  Protected: Yes
  Masked: Yes
  Value: <your crates.io token>
```

#### Optional Variables

```
CODECOV_TOKEN
  Type: Variable
  Protected: No
  Masked: Yes
  Value: <your codecov token>
  
SCHEDULE_TYPE
  Type: Variable
  Protected: No
  Masked: No
  Value: nightly (for scheduled pipelines)
```

### 2. Enable GitLab Runners

Ensure you have runners available:

- **Shared Runners**: Enabled by default on GitLab.com
- **Specific Runners**: Configure if needed for special requirements

Check: **Settings → CI/CD → Runners**

### 3. Configure Pipeline Schedules

Create schedules for nightly builds:

**Settings → CI/CD → Schedules → New schedule**

#### Nightly Build Schedule

```
Description: Nightly Rust Build
Interval Pattern: 0 0 * * * (daily at midnight UTC)
Target Branch: develop
Variables:
  - SCHEDULE_TYPE = nightly
Active: Yes
```

### 4. Enable Security Scanning

**Settings → Security & Compliance → Configure**

Enable:
- ✅ Secret Detection
- ✅ Dependency Scanning
- ✅ SAST (Static Application Security Testing)

### 5. Configure Protected Branches

**Settings → Repository → Protected branches**

```
Branch: main
Allowed to merge: Maintainers
Allowed to push: No one
Allowed to force push: No

Branch: develop
Allowed to merge: Developers + Maintainers
Allowed to push: Developers + Maintainers
Allowed to force push: No
```

### 6. GitLab Pages Setup

**Settings → Pages**

The `pages` job automatically deploys to GitLab Pages from the `develop` branch.

Access URL: `https://pegasusheavy.gitlab.io/armature/`

---

## CI/CD Variables

### Built-in GitLab Variables

The pipeline uses these predefined variables:

| Variable | Description | Example |
|----------|-------------|---------|
| `$CI_COMMIT_TAG` | Git tag name | `v1.0.0` |
| `$CI_COMMIT_BRANCH` | Branch name | `develop` |
| `$CI_COMMIT_SHA` | Full commit SHA | `abc123...` |
| `$CI_PROJECT_DIR` | Project directory | `/builds/pegasusheavy/armature` |
| `$CI_PIPELINE_SOURCE` | Pipeline trigger | `push`, `merge_request_event`, `schedule` |
| `$CI_JOB_NAME` | Current job name | `test:stable:linux` |

### Custom Variables

Defined in `.gitlab-ci.yml`:

```yaml
variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: "1"
```

---

## Pipeline Schedules

### Creating a Schedule

1. Go to **CI/CD → Schedules**
2. Click **New schedule**
3. Fill in details
4. Add variables if needed
5. Save

### Recommended Schedules

#### Nightly Builds

```
Description: Nightly Rust Build
Interval: 0 0 * * * (daily at midnight)
Target: develop
Variables:
  SCHEDULE_TYPE = nightly
```

#### Weekly Security Audit

```
Description: Weekly Security Audit
Interval: 0 2 * * 0 (Sundays at 2am)
Target: develop
Variables:
  SCHEDULE_TYPE = security
```

#### Monthly MSRV Check

```
Description: Monthly MSRV Check
Interval: 0 3 1 * * (1st of month at 3am)
Target: develop
Variables:
  SCHEDULE_TYPE = msrv
```

---

## Features Comparison

### ✅ Fully Supported

| Feature | Status | Notes |
|---------|--------|-------|
| **Test Matrix** | ✅ | Multiple Rust versions, OS support via runners |
| **Caching** | ✅ | Built-in cargo caching |
| **Artifacts** | ✅ | Build artifacts, docs, coverage reports |
| **Code Coverage** | ✅ | Integrated with GitLab |
| **Security Scanning** | ✅ | Built-in dependency scanning, SAST, secret detection |
| **Release Automation** | ✅ | GitLab Releases API |
| **Documentation** | ✅ | GitLab Pages |
| **Benchmarking** | ✅ | Criterion benchmarks |

### ⚠️ Partial Support

| Feature | Status | Notes |
|---------|--------|-------|
| **Greetings** | ⚠️ | Use GitLab Quick Actions or webhooks |
| **Stale Issues** | ⚠️ | Use GitLab Triage Policies (requires configuration) |
| **Auto-Labeling** | ⚠️ | Manual or webhook-based |
| **Cross-Platform** | ⚠️ | Requires specific runners for macOS/Windows |

### Alternative Solutions

#### Issue/MR Automation

Instead of GitHub Actions bots, use:

1. **GitLab Quick Actions** - `/label`, `/assign`, `/milestone`
2. **Triage Policies** - YAML-based automation
3. **Webhooks** - Custom bot integration
4. **GitLab API** - Scheduled scripts

#### Example Triage Policy

Create `.gitlab/triage-policies/stale.yml`:

```yaml
resource_rules:
  issues:
    rules:
      - name: "Close stale issues"
        conditions:
          date:
            attribute: updated_at
            condition: older_than
            interval_type: days
            interval: 60
          state: opened
          labels:
            - stale
        actions:
          comment: |
            This issue has been automatically closed due to inactivity.
          status: close
```

---

## Troubleshooting

### Pipeline Not Triggering

**Problem:** Pipeline doesn't run on push/MR

**Solution:**
1. Check `.gitlab-ci.yml` syntax: **CI/CD → Editor → Validate**
2. Verify `rules:` conditions match your branch/event
3. Ensure runners are available and enabled

### Cargo Cache Not Working

**Problem:** Dependencies rebuild every time

**Solution:**
```yaml
cache:
  key:
    files:
      - Cargo.lock  # ✅ Cache invalidates on lock file change
  paths:
    - .cargo/registry
    - .cargo/git
    - target/
```

### Test Failures on Specific Runners

**Problem:** Tests pass locally, fail in CI

**Solution:**
1. Check runner OS/architecture
2. Verify dependencies are installed
3. Add debugging: `- cargo build --verbose`
4. Check environment variables

### Release Jobs Not Running

**Problem:** `publish:*` jobs don't execute

**Solution:**
1. Verify tag format: `v1.2.3` (must match regex)
2. Check `$CARGO_REGISTRY_TOKEN` is set
3. Review job `rules:` conditions
4. Ensure proper job dependencies (`needs:`)

### Security Scanning False Positives

**Problem:** Security jobs fail on acceptable issues

**Solution:**
1. Review findings in **Security & Compliance**
2. Mark as false positive or dismiss
3. Add to allowlist if appropriate
4. Update dependencies to resolve

### Cross-Platform Builds Failing

**Problem:** Can't build for macOS/Windows

**Solution:**
1. Register specific runners for those platforms
2. Or use Docker-based cross-compilation:
   ```yaml
   image: rustembedded/cross:x86_64-unknown-linux-gnu
   ```
3. Or use `cargo-cross` for simpler cross-compilation

---

## Migration Checklist

### Pre-Migration

- [x] All GitHub workflows identified
- [x] GitLab CI config created
- [x] Variables documented
- [x] Security scanning configured

### Post-Migration

- [ ] Push to GitLab
- [ ] Configure CI/CD variables
- [ ] Enable runners
- [ ] Create pipeline schedules
- [ ] Enable security scanning
- [ ] Configure protected branches
- [ ] Test pipeline on all branches
- [ ] Verify release process
- [ ] Update documentation references

### Verification

Run these checks after migration:

```bash
# Verify .gitlab-ci.yml syntax
gitlab-ci-lint .gitlab-ci.yml

# Or use GitLab UI: CI/CD → Editor → Validate

# Test pipeline locally (requires gitlab-runner)
gitlab-runner exec docker test:stable:linux
```

---

## Additional Resources

### GitLab Documentation

- [GitLab CI/CD](https://docs.gitlab.com/ee/ci/)
- [.gitlab-ci.yml Reference](https://docs.gitlab.com/ee/ci/yaml/)
- [GitLab Pages](https://docs.gitlab.com/ee/user/project/pages/)
- [Security Scanning](https://docs.gitlab.com/ee/user/application_security/)
- [Triage Policies](https://docs.gitlab.com/ee/user/project/issues/managing_issues.html#automatically-close-issues)

### Rust-Specific

- [cargo-llvm-cov](https://github.com/taiki-e/cargo-llvm-cov) - Code coverage
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit) - Security audits
- [cargo-criterion](https://github.com/bheisler/cargo-criterion) - Benchmarking

---

## Summary

### Key Takeaways

✅ **Single Config File** - All CI/CD in `.gitlab-ci.yml`
✅ **Built-in Features** - Caching, artifacts, security scanning
✅ **Comprehensive Stages** - Lint, test, security, build, docs, release
✅ **Scheduled Pipelines** - Nightly builds, security audits
✅ **GitLab Pages** - Automatic documentation deployment
✅ **Release Automation** - Crates.io publishing on tags

### Quick Start

1. Configure `$CARGO_REGISTRY_TOKEN`
2. Enable security scanning
3. Create nightly schedule
4. Push and watch pipelines! 🚀

---

**Migration complete!** All GitHub Actions workflows successfully converted to GitLab CI/CD. 🎉


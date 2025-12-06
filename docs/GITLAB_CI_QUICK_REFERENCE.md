# GitLab CI/CD Quick Reference

Quick reference card for common GitLab CI/CD tasks in the Armature project.

## 🚀 Quick Start

### Trigger Pipeline

```bash
# Push to trigger pipeline
git push origin develop

# Create MR to trigger pipeline
git push origin feature/my-feature
# Then create MR in GitLab UI
```

### View Pipelines

```bash
# Using glab CLI
glab ci status
glab ci view

# Or in browser
https://gitlab.com/pegasusheavy/armature/-/pipelines
```

---

## 📋 Common Tasks

### Run Pipeline Locally

```bash
# Install gitlab-runner
curl -L https://gitlab-runner-downloads.s3.amazonaws.com/latest/binaries/gitlab-runner-linux-amd64 -o gitlab-runner
chmod +x gitlab-runner

# Run specific job locally
gitlab-runner exec docker test:stable:linux --docker-image=rust:latest
```

### Validate Configuration

```bash
# Using glab CLI
glab ci lint

# Or in browser
https://gitlab.com/pegasusheavy/armature/-/ci/lint
```

### Retry Failed Jobs

```bash
# Using glab CLI
glab ci retry <pipeline-id>

# Or in browser
Go to pipeline → Click "Retry" button
```

### Download Artifacts

```bash
# Using glab CLI
glab ci artifact <job-name>

# Or via API
curl --header "PRIVATE-TOKEN: <your_token>" \
  "https://gitlab.com/api/v4/projects/pegasusheavy%2Farmature/jobs/<job-id>/artifacts"
```

---

## 🔧 Configuration

### Add CI/CD Variable

```bash
# Using glab CLI
glab variable set CARGO_REGISTRY_TOKEN "your-token" --masked

# Or in browser
Settings → CI/CD → Variables → Add Variable
```

### Create Pipeline Schedule

```bash
# In browser only
Settings → CI/CD → Schedules → New schedule

# Configure:
Description: Nightly Build
Interval: 0 0 * * * (cron syntax)
Target: develop
Variables: SCHEDULE_TYPE=nightly
```

### View Runner Status

```bash
# Using glab CLI
glab ci runners

# Or in browser
Settings → CI/CD → Runners
```

---

## 📊 Pipeline Stages

### Stage Order

```
1. lint      → Format & clippy checks
2. test      → Tests, doc tests, examples
3. security  → Audits & scanning
4. build     → Release builds
5. benchmark → Performance tests
6. docs      → Documentation generation
7. release   → Publishing & releases
```

### Key Jobs

| Job | Stage | Runs On | Purpose |
|-----|-------|---------|---------|
| `fmt:check` | lint | MR, develop, main | Code formatting |
| `clippy` | lint | MR, develop, main | Linting |
| `test:stable:linux` | test | MR, develop, main | Unit tests |
| `doc-tests` | test | MR, develop, main | Documentation tests |
| `security:audit` | security | MR, develop, main | Dependency audit |
| `coverage` | test | develop, main | Code coverage |
| `build:release` | build | develop, main, tags | Release build |
| `benchmarks` | benchmark | develop, MR | Performance |
| `docs:build` | docs | develop, main | API docs |
| `pages` | docs | develop | GitLab Pages |
| `release:create` | release | tags | Create release |
| `publish:*` | release | tags | Publish to crates.io |

---

## 🏷️ Triggering Rules

### By Branch

```yaml
# Develop branch
rules:
  - if: $CI_COMMIT_BRANCH == "develop"

# Main branch
rules:
  - if: $CI_COMMIT_BRANCH == "main"

# Any branch
rules:
  - if: $CI_COMMIT_BRANCH
```

### By Event

```yaml
# Merge requests
rules:
  - if: $CI_PIPELINE_SOURCE == "merge_request_event"

# Tags
rules:
  - if: $CI_COMMIT_TAG

# Scheduled pipelines
rules:
  - if: $CI_PIPELINE_SOURCE == "schedule"

# Manual trigger
rules:
  - if: $CI_PIPELINE_SOURCE == "web"
```

### Combined Rules

```yaml
rules:
  - if: $CI_PIPELINE_SOURCE == "merge_request_event"
  - if: $CI_COMMIT_BRANCH == "develop"
  - if: $CI_COMMIT_BRANCH == "main"
```

---

## 🔍 Debugging

### View Job Logs

```bash
# Using glab CLI
glab ci trace <job-id>

# Or in browser
Pipeline → Click job → View logs
```

### Enable Debug Logging

Add to `.gitlab-ci.yml`:

```yaml
variables:
  CI_DEBUG_TRACE: "true"
```

Or set as CI/CD variable for temporary debugging.

### Common Issues

#### Cache Not Working

```yaml
# Fix: Use correct cache key
cache:
  key:
    files:
      - Cargo.lock
  paths:
    - .cargo/registry
    - target/
```

#### Job Stuck in Pending

- Check runner availability
- Verify runner tags match job tags
- Check runner capacity

#### Tests Failing in CI

```yaml
# Add verbose output
script:
  - cargo test --verbose --all-features
  - cargo test --doc --verbose
```

---

## 📦 Artifacts

### Available Artifacts

| Job | Artifact | Expires |
|-----|----------|---------|
| `build:release` | Release binary | 7 days |
| `doc-tests` | Generated docs | 30 days |
| `coverage` | Coverage report | 30 days |
| `docs:build` | API documentation | 30 days |
| `benchmarks` | Criterion results | 30 days |

### Download Artifact

```bash
# Via glab
glab ci artifact download <job-name>

# Via browser
Pipeline → Job → Browse → Download
```

---

## 🔐 Security

### Security Scanning Jobs

```
- secret_detection           → Find exposed secrets
- gemnasium-dependency_scanning  → Dependency vulnerabilities
- sast                       → Static analysis
```

### View Security Dashboard

```
Security & Compliance → Security Dashboard
```

### Resolve Security Finding

```
Security & Compliance → Vulnerability Report
→ Select finding → Dismiss or Create issue
```

---

## 🚢 Releases

### Create Release

```bash
# Create and push tag
git tag -a v1.0.0 -m "Release v1.0.0"
git push origin v1.0.0

# Pipeline automatically:
# 1. Creates GitLab release
# 2. Publishes to crates.io
```

### Release Requirements

- Tag format: `v*.*.*` (e.g., `v1.0.0`)
- Tag must be on develop or main branch
- All tests must pass
- `$CARGO_REGISTRY_TOKEN` must be set

### Publish Order

```
1. armature-core
2. armature-macro
3. All other crates (parallel)
4. armature (main crate, last)
```

---

## 📖 Documentation

### Build Docs Locally

```bash
# Build API docs
cargo doc --all-features --no-deps --open

# Run doc tests
cargo test --doc --all-features

# Check for missing docs
cargo doc --all --all-features --no-deps 2>&1 | grep "warning: missing"
```

### Deploy to GitLab Pages

Automatically deployed on push to `develop`:
- URL: https://pegasusheavy.gitlab.io/armature/
- Job: `pages`
- Source: `web/` directory

---

## ⏱️ Pipeline Timing

### Typical Durations

| Stage | Duration | Can Fail? |
|-------|----------|-----------|
| Lint | ~2 min | Yes |
| Test | ~10 min | Yes |
| Security | ~3 min | No (audit only) |
| Build | ~5 min | Yes |
| Benchmark | ~15 min | No |
| Docs | ~3 min | No |
| Release | ~20 min | No (on error) |

**Total:** ~40-60 minutes for full pipeline

---

## 🎯 Best Practices

### 1. Branch Naming

```
feature/add-new-module
bugfix/fix-memory-leak
hotfix/critical-security-fix
release/v1.0.0
```

### 2. Commit Messages

```
feat: add new authentication module
fix: resolve memory leak in cache
docs: update API documentation
chore: update dependencies
```

### 3. MR Workflow

1. Create feature branch from `develop`
2. Make changes and commit
3. Push and create MR
4. Wait for pipeline to pass
5. Request review
6. Merge when approved

### 4. Testing Locally First

```bash
# Before pushing
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test --all-features
cargo test --doc
```

---

## 🔗 Useful Links

### GitLab Resources

- [Pipeline](https://gitlab.com/pegasusheavy/armature/-/pipelines)
- [Jobs](https://gitlab.com/pegasusheavy/armature/-/jobs)
- [Schedules](https://gitlab.com/pegasusheavy/armature/-/pipeline_schedules)
- [Variables](https://gitlab.com/pegasusheavy/armature/-/settings/ci_cd)
- [Security](https://gitlab.com/pegasusheavy/armature/-/security/dashboard)
- [Pages](https://pegasusheavy.gitlab.io/armature/)

### Documentation

- [GitLab CI/CD Docs](https://docs.gitlab.com/ee/ci/)
- [.gitlab-ci.yml Reference](https://docs.gitlab.com/ee/ci/yaml/)
- [GitLab CLI (glab)](https://gitlab.com/gitlab-org/cli)

---

## 💡 Tips & Tricks

### Speed Up Pipelines

```yaml
# Use smaller Docker images
image: rust:slim

# Limit test matrix
test:stable:linux:  # Only stable on Linux for MRs
  rules:
    - if: $CI_PIPELINE_SOURCE == "merge_request_event"
```

### Manual Jobs

```yaml
deploy:production:
  stage: deploy
  when: manual  # Requires manual trigger
  script:
    - ./deploy.sh
```

### Conditional Jobs

```yaml
expensive:job:
  script:
    - cargo bench --all-features
  rules:
    - if: $CI_COMMIT_MESSAGE =~ /\[bench\]/  # Only if commit message contains [bench]
```

### Job Dependencies

```yaml
job_b:
  needs:
    - job_a  # Wait for job_a to complete first
  script:
    - echo "Running after job_a"
```

---

## 🆘 Getting Help

### Check Pipeline Status

```bash
# View current status
glab ci status

# View specific pipeline
glab ci view <pipeline-id>

# View logs
glab ci trace <job-id>
```

### Troubleshooting

1. Check `.gitlab-ci.yml` syntax: CI/CD → Editor → Validate
2. Review job logs for errors
3. Verify variables are set correctly
4. Check runner availability
5. Review GitLab CI/CD documentation

---

**Quick reference complete!** 🎉


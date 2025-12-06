# Deployment Scripts

Scripts for managing Armature releases and version synchronization.

## Scripts

### `sync-versions.sh`

Synchronizes versions across all workspace crates.

**Usage:**

```bash
# Check current versions
./scripts/sync-versions.sh

# Update all versions
./scripts/sync-versions.sh 0.2.0
```

**Features:**
- Lists all workspace crate versions
- Identifies out-of-sync versions
- Updates all Cargo.toml files atomically
- Validates semver format

### `deploy.sh`

Comprehensive deployment script that handles the entire release process.

**Usage:**

```bash
# Full deployment with tests
./scripts/deploy.sh 0.2.0

# Skip tests (faster, use with caution)
./scripts/deploy.sh 0.2.0 --skip-tests
```

**Process:**
1. ✓ Check git status (must be clean)
2. ✓ Sync versions across all crates
3. ✓ Format code (`cargo fmt`)
4. ✓ Run clippy (`cargo clippy`)
5. ✓ Run tests (`cargo test --all`)
6. ✓ Build release (`cargo build --release`)
7. ✓ Create git commit and tag

**Output:**
- Commit: `chore: bump version to X.Y.Z`
- Tag: `vX.Y.Z`

## Typical Workflow

### 1. Prepare Release

```bash
# Ensure you're on develop branch
git checkout develop
git pull origin develop

# Run deployment script
./scripts/deploy.sh 0.2.0
```

### 2. Review Changes

```bash
# Review the commit
git show

# Review the tag
git tag -v v0.2.0
```

### 3. Push to Remote

```bash
# Push commit and tag
git push origin develop
git push origin v0.2.0

# Or push all tags
git push --tags
```

### 4. Create GitHub Release

```bash
# Using GitHub CLI
gh release create v0.2.0 --generate-notes

# Or manually on GitHub
# Go to: https://github.com/quinnjr/armature/releases/new
```

### 5. Publish to crates.io

```bash
# Publish all crates (must be done in order due to dependencies)
cd armature-core && cargo publish
cd ../armature-macro && cargo publish
cd ../armature-graphql && cargo publish
# ... etc for each crate

# Or use cargo-workspaces
cargo workspaces publish --from-git
```

## Version Format

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR**: Breaking changes (e.g., `1.0.0 → 2.0.0`)
- **MINOR**: New features, backwards compatible (e.g., `1.0.0 → 1.1.0`)
- **PATCH**: Bug fixes, backwards compatible (e.g., `1.0.0 → 1.0.1`)
- **Pre-release**: Alpha, beta, RC (e.g., `1.0.0-beta.1`)

## Troubleshooting

### Uncommitted Changes

**Error:** `Working directory has uncommitted changes`

**Solution:**
```bash
# Option 1: Commit changes
git add -A
git commit -m "feat: your changes"

# Option 2: Stash changes
git stash
./scripts/deploy.sh 0.2.0
git stash pop
```

### Tests Failing

**Error:** `Tests failed`

**Solution:**
```bash
# Fix tests first
cargo test --all

# Or skip tests (not recommended)
./scripts/deploy.sh 0.2.0 --skip-tests
```

### Undo Deployment

If you need to undo a deployment before pushing:

```bash
# Remove commit and tag
git reset --hard HEAD~1
git tag -d v0.2.0
```

### Crates.io Publication Order

Due to dependencies, publish in this order:

1. `armature-core`
2. `armature-macro`
3. All other crates (in any order)
4. `armature` (main crate, last)

## CI/CD Integration

These scripts are designed to work with GitHub Actions:

```yaml
# .github/workflows/release.yml
- name: Deploy
  run: ./scripts/deploy.sh ${{ github.ref_name }}
```

## Requirements

- **Git**: Clean working directory
- **Rust**: 1.85+ with `cargo`, `rustfmt`, `clippy`
- **Bash**: 4.0+
- **sed**: GNU sed (for version replacement)

## Notes

- Always test on staging/development before production
- Review CHANGELOG.md before releasing
- Ensure all documentation is up to date
- Update README.md if API changes


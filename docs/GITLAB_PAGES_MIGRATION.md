# GitLab Pages Migration Guide

Complete guide for migrating the Armature documentation website from GitHub Pages to GitLab Pages.

## Table of Contents

- [Overview](#overview)
- [What Changed](#what-changed)
- [Branch Structure](#branch-structure)
- [GitLab Pages Setup](#gitlab-pages-setup)
- [Deployment Process](#deployment-process)
- [Accessing the Site](#accessing-the-site)
- [Troubleshooting](#troubleshooting)

---

## Overview

The Armature documentation website has been migrated from GitHub Pages to GitLab Pages while maintaining the same branch structure (`gh-pages` branch).

### Before Migration

- **Hosting:** GitHub Pages
- **URL:** https://quinnjr.github.io/armature/
- **Deployment:** Manual push to `gh-pages` branch
- **Build:** Manual or GitHub Actions

### After Migration

- **Hosting:** GitLab Pages
- **URL:** https://pegasusheavy.gitlab.io/armature/
- **Deployment:** Automatic via GitLab CI/CD
- **Build:** Automatic on push to `gh-pages` or `develop`

---

## What Changed

### Website Technology Stack

The website remains the same:
- ✅ **Angular 21+** - Latest version
- ✅ **Tailwind CSS 4+** - CSS-first configuration
- ✅ **TypeScript 5.7+**
- ✅ **pnpm** - Package manager

**No changes needed to the application code!**

### URLs Updated

| Item | Old (GitHub) | New (GitLab) |
|------|-------------|--------------|
| **Live Site** | quinnjr.github.io/armature | pegasusheavy.gitlab.io/armature |
| **Repository** | github.com/quinnjr/armature | gitlab.com/pegasusheavy/armature |
| **Author** | Joseph R. Quinn | Pegasus Heavy Industries LLC |

### Files Updated

**On `gh-pages` branch:**
- ✅ `web/package.json` - Repository and homepage URLs
- ✅ `web/README.md` - Deployment instructions
- ✅ `README.md` - Live site URL, source links

**On `develop` branch:**
- ✅ `.gitlab-ci.yml` - Pages job configuration

---

## Branch Structure

### Repository Branches

```
armature/
├── develop (main development branch)
│   ├── Rust source code
│   ├── Documentation (docs/)
│   └── .gitlab-ci.yml (CI/CD configuration)
│
└── gh-pages (documentation website)
    ├── web/ (Angular 21 app)
    │   ├── src/
    │   ├── public/
    │   └── package.json
    ├── README.md
    └── LICENSE
```

### Why Keep `gh-pages` Name?

Even though we're migrating to GitLab, the branch remains named `gh-pages` for:
- ✅ **Consistency** - Familiar to contributors
- ✅ **Convention** - Common name for static site branches
- ✅ **No Breaking Changes** - Existing workflows still work

---

## GitLab Pages Setup

### Automatic Configuration

GitLab Pages is automatically configured via the `pages` job in `.gitlab-ci.yml`:

```yaml
pages:
  stage: docs
  image: node:20
  before_script:
    - corepack enable
    - corepack prepare pnpm@latest --activate
    - git config --global user.email "ci@gitlab.com"
    - git config --global user.name "GitLab CI"
  script:
    # Fetch and checkout gh-pages branch
    - git fetch origin gh-pages
    - git checkout gh-pages
    - cd web
    # Install dependencies
    - pnpm install
    # Build Angular app for production
    - pnpm run build
    - cd ..
    # Prepare public directory for GitLab Pages
    - mkdir -p public
    - cp -r web/dist/web/browser/* public/
    # Add .nojekyll to disable Jekyll processing
    - touch public/.nojekyll
    # Return to original branch
    - git checkout -
  artifacts:
    paths:
      - public
  rules:
    - if: $CI_COMMIT_BRANCH == "develop"
    - if: $CI_COMMIT_BRANCH == "gh-pages"
```

### How It Works

1. **Trigger:** Push to `develop` or `gh-pages` branch
2. **Checkout:** CI fetches `gh-pages` branch
3. **Build:** Installs dependencies and builds Angular app
4. **Deploy:** Copies build output to `public/` directory
5. **Publish:** GitLab Pages serves from `public/` directory

### No Configuration Required!

GitLab automatically:
- ✅ Detects the `pages` job
- ✅ Publishes `public/` directory
- ✅ Generates the URL: `https://pegasusheavy.gitlab.io/armature/`

---

## Deployment Process

### Automatic Deployment

The site automatically rebuilds and deploys when:

#### Trigger 1: Push to `gh-pages` Branch

```bash
# Make changes to website
git checkout gh-pages
cd web
# Edit files...

# Commit and push
git add .
git commit -m "Update documentation"
git push origin gh-pages
```

**Result:** GitLab CI builds and deploys the site.

#### Trigger 2: Push to `develop` Branch

```bash
# Make changes to framework
git checkout develop
# Edit files...

# Commit and push
git add .
git commit -m "Update feature"
git push origin develop
```

**Result:** GitLab CI rebuilds documentation site (in case it references latest framework features).

### Manual Deployment

You can also trigger manually:

#### Via GitLab UI

1. Go to **CI/CD → Pipelines**
2. Click **Run pipeline**
3. Select branch: `develop` or `gh-pages`
4. Click **Run pipeline**

#### Via glab CLI

```bash
# Trigger pipeline on develop
glab ci trigger -b develop

# Trigger pipeline on gh-pages
glab ci trigger -b gh-pages
```

---

## Accessing the Site

### Production URL

🌐 **Live Site:** https://pegasusheavy.gitlab.io/armature/

### Verification

After deployment (usually 1-3 minutes), verify:

```bash
# Check if site is live
curl -I https://pegasusheavy.gitlab.io/armature/

# Expected response
HTTP/2 200
content-type: text/html
```

### GitLab Pages Settings

View deployment status:
- **Settings → Pages**
- Shows: URL, deployment status, last deployment time

---

## Local Development

### Running Locally

```bash
# Switch to gh-pages branch
git checkout gh-pages

# Navigate to web directory
cd web

# Install dependencies
pnpm install

# Start development server
pnpm start
```

**Access:** http://localhost:4200

### Building Locally

```bash
# Build for production
pnpm run build

# Output
dist/web/browser/  (or dist/web/)
```

### Testing Build

```bash
# Install a simple HTTP server
npm install -g http-server

# Serve the build
cd dist/web/browser  # or dist/web
http-server -p 8080

# Access
http://localhost:8080
```

---

## Troubleshooting

### Site Not Updating

**Problem:** Changes pushed but site not updated

**Solutions:**

1. **Check Pipeline Status:**
   ```bash
   glab ci status
   ```
   Or visit: https://gitlab.com/pegasusheavy/armature/-/pipelines

2. **View Job Logs:**
   - Go to pipeline
   - Click `pages` job
   - Review logs for errors

3. **Verify Branch:**
   ```bash
   git branch --show-current
   # Should be: gh-pages or develop
   ```

4. **Trigger Manual Rebuild:**
   ```bash
   glab ci trigger -b develop
   ```

### 404 Error

**Problem:** Site shows 404 Not Found

**Solutions:**

1. **Check GitLab Pages is Enabled:**
   - Settings → Pages
   - Ensure Pages is enabled

2. **Verify Public Directory:**
   ```bash
   # In CI logs, check:
   # - public/ directory exists
   # - index.html is present
   ```

3. **Check .nojekyll:**
   ```bash
   # Ensure .nojekyll file exists in public/
   - touch public/.nojekyll
   ```

### Build Fails

**Problem:** `pages` job fails during build

**Solutions:**

1. **Check Node Version:**
   ```yaml
   # Ensure using Node 20
   image: node:20
   ```

2. **Verify Dependencies:**
   ```bash
   # Test locally
   cd web
   pnpm install
   pnpm run build
   ```

3. **Check pnpm Version:**
   ```bash
   # In CI logs, verify
   corepack prepare pnpm@latest --activate
   pnpm --version
   ```

4. **Review Build Output:**
   ```bash
   # Check if dist/ directory is created
   ls -la web/dist/web/
   ```

### Styles Not Loading

**Problem:** Site loads but no styles

**Solutions:**

1. **Check Base Href:**
   ```bash
   # In angular.json, verify
   "baseHref": "/"
   # NOT: "/armature/"
   ```

2. **Verify Build Output:**
   ```bash
   # Check for CSS files in dist/
   ls -la web/dist/web/browser/*.css
   ```

3. **Clear Cache:**
   - Hard refresh browser (Ctrl+Shift+R / Cmd+Shift+R)
   - Clear browser cache

### Permission Denied

**Problem:** Cannot push to `gh-pages` branch

**Solutions:**

1. **Check Branch Protection:**
   - Settings → Repository → Protected Branches
   - Ensure `gh-pages` allows pushes

2. **Verify Permissions:**
   - Settings → Members
   - Ensure you have Developer or Maintainer role

3. **Check Remote:**
   ```bash
   git remote -v
   # Should show gitlab.com
   ```

---

## Migration Checklist

### Completed ✅

- [x] Updated `web/package.json` URLs
- [x] Updated `web/README.md` deployment instructions
- [x] Updated `README.md` on `gh-pages` branch
- [x] Configured GitLab Pages job in `.gitlab-ci.yml`
- [x] Set up automatic deployment triggers
- [x] Created migration documentation

### Post-Migration Tasks

- [ ] Push both branches to GitLab
- [ ] Verify site builds successfully
- [ ] Test deployment pipeline
- [ ] Update any external links to old URL
- [ ] Update README badges if any
- [ ] Notify team of new URL

---

## Key Differences: GitHub Pages vs GitLab Pages

| Feature | GitHub Pages | GitLab Pages |
|---------|--------------|--------------|
| **URL Format** | `username.github.io/repo` | `namespace.gitlab.io/repo` |
| **Build** | Jekyll (automatic) | CI/CD pipeline (configurable) |
| **Deployment** | Push to `gh-pages` | CI/CD job artifact |
| **Custom Domain** | CNAME file | Settings → Pages |
| **HTTPS** | Automatic | Automatic |
| **Build Time** | Limited | Configurable |
| **Cache** | Limited | Full control |
| **Private Repos** | GitHub Pro+ | Free on GitLab |

---

## Advanced Configuration

### Custom Domain

To use a custom domain:

1. **Add CNAME Record:**
   ```
   Type: CNAME
   Name: docs (or @)
   Value: pegasusheavy.gitlab.io
   ```

2. **Configure in GitLab:**
   - Settings → Pages
   - Click **New Domain**
   - Enter: `docs.yourdomai.com`
   - Verify DNS

3. **Update `.gitlab-ci.yml`:**
   ```yaml
   pages:
     script:
       # ... existing build steps
       - echo "docs.yourdomain.com" > public/CNAME
   ```

### Multiple Branches

Deploy different branches to different URLs:

```yaml
pages:
  # Production (main site)
  rules:
    - if: $CI_COMMIT_BRANCH == "develop"

pages:staging:
  # Staging environment
  stage: docs
  script:
    # Same build process
    - ...
  artifacts:
    paths:
      - public-staging
  rules:
    - if: $CI_COMMIT_BRANCH == "staging"
```

---

## Summary

### What You Need to Know

✅ **URL Changed:** quinnjr.github.io/armature → pegasusheavy.gitlab.io/armature
✅ **Automatic Deployment:** Push to `gh-pages` or `develop` triggers rebuild
✅ **No Code Changes:** Angular app works exactly the same
✅ **Better Control:** Full CI/CD pipeline control
✅ **Free Hosting:** GitLab Pages free for all repos

### Quick Commands

```bash
# Update gh-pages branch
git checkout gh-pages
# Make changes...
git push origin gh-pages

# View pipeline
glab ci status

# Access site
https://pegasusheavy.gitlab.io/armature/
```

---

**Migration complete!** The documentation website is now hosted on GitLab Pages with automatic CI/CD deployment. 🚀


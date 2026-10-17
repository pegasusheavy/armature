# Kubernetes Manifests

Kubernetes templates, with production-oriented defaults (non-root, resource limits, health probes), for deploying Armature applications.

## Quick Start

The base deliberately ships **no image and no Secret**:

- `deployment.yaml` uses the placeholder image `REPLACE_WITH_YOUR_IMAGE`,
  which is not a valid reference, so pods fail with `InvalidImageName` until
  an overlay sets your own image (pinned by digest). The Armature project does
  not publish an application image.
- The `armature-api-secrets` Secret (key `jwt-secret`, optionally
  `database-url` / `redis-url`) must be created separately; see
  `secret.yaml.example`. Until it exists, pods stay in
  `CreateContainerConfigError` instead of running with a default JWT key.

```bash
# 1. Create the Secret from freshly generated values
kubectl create namespace armature
kubectl -n armature create secret generic armature-api-secrets \
  --from-literal=jwt-secret="$(openssl rand -hex 32)"

# 2. Apply an overlay that sets the image (see "Using Kustomize" below)
kubectl apply -k overlays/production
```

## Files

| File | Description |
|------|-------------|
| `namespace.yaml` | Kubernetes namespace |
| `deployment.yaml` | Main application deployment |
| `service.yaml` | ClusterIP service |
| `ingress.yaml` | Ingress with TLS |
| `hpa.yaml` | Horizontal Pod Autoscaler |
| `pdb.yaml` | Pod Disruption Budget |
| `configmap.yaml` | Application configuration |
| `secret.yaml.example` | Secret shape and how to create it (not applied) |
| `serviceaccount.yaml` | Service account |
| `networkpolicy.yaml` | Network security policies |
| `kustomization.yaml` | Kustomize base |

## Customization

These manifests are plain, template-free YAML with static defaults (app name `armature-api`, namespace `armature`, container port `3000`, metrics port `9090`, placeholder image `REPLACE_WITH_YOUR_IMAGE`). They are syntactically validated in CI. Copy them into your repository and customize them with:

- **Kustomize**: Use overlays to patch values (see below) — this is the recommended approach
- **Helm**: See `../helm/armature/` for a fully parameterized Helm chart instead
- **Manual edits**: Just edit the YAML directly for one-off changes

### Using Kustomize

Create an overlay:

```yaml
# overlays/production/kustomization.yaml
apiVersion: kustomize.config.k8s.io/v1beta1
kind: Kustomization

resources:
  - ../../k8s   # a local copy, not a remote base

namespace: production

images:
  - name: REPLACE_WITH_YOUR_IMAGE
    newName: registry.example.com/my-api
    digest: sha256:<digest printed by docker push>

patches:
  - patch: |-
      - op: replace
        path: /spec/replicas
        value: 5
    target:
      kind: Deployment
      name: armature-api
```

```bash
kubectl apply -k overlays/production
```

## Security Features

- ✅ Non-root user (UID 1000)
- ✅ Read-only root filesystem
- ✅ Dropped capabilities
- ✅ Network policy included (default-deny, explicit allows; metrics port reachable from `monitoring` only)
- ✅ `seccompProfile: RuntimeDefault`
- ✅ No default image or Secret (fail closed until you provide them)
- ✅ Service account without auto-mount
- ✅ Resource limits

## Health Checks

All deployments include:

- **Liveness probe**: `/health/live`
- **Readiness probe**: `/health/ready`
- **Startup probe**: `/health/live` (for slow starts)

## Monitoring

Deployments are annotated for Prometheus scraping:

```yaml
prometheus.io/scrape: "true"
prometheus.io/port: "3000"
prometheus.io/path: "/metrics"
```


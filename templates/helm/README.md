# Armature Helm Chart

Production-ready Helm chart for deploying Armature applications on Kubernetes.

## Quick Start

The chart has **no default image and no default JWT secret**: `helm install`
fails until you provide them. (The Armature project does not publish an
application image; build your own and push it to a registry you control.)

```bash
# Secret managed outside the chart (recommended): must contain the key
# jwt-secret, and optionally database-url / redis-url.
kubectl create secret generic my-api-secrets \
  --from-literal=jwt-secret="$(openssl rand -hex 32)"

helm install my-api ./armature \
  --set image.repository=registry.example.com/my-api \
  --set image.digest=sha256:<digest printed by docker push> \
  --set secrets.existingSecret=my-api-secrets \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=api.example.com

# Or let the chart create the Secret (keep the value out of committed files)
helm install my-api ./armature \
  --set image.repository=registry.example.com/my-api \
  --set image.tag=v1.0.0 \
  --set secrets.jwtSecret="$(openssl rand -hex 32)"
```

`image.tag=latest` is rejected; prefer `image.digest`. Linting needs values
too: `helm lint ./armature -f ./armature/ci/lint-values.yaml`.

## Configuration

See `armature/values.yaml` for all available options.

### Common Configurations

#### Basic Deployment

```yaml
# values-basic.yaml
replicaCount: 2
image:
  repository: registry.example.com/my-api
  digest: sha256:<digest>
resources:
  requests:
    cpu: 100m
    memory: 64Mi
  limits:
    cpu: 500m
    memory: 256Mi
```

#### With Ingress

```yaml
# values-ingress.yaml
ingress:
  enabled: true
  className: nginx
  hosts:
    - host: api.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: api-tls
      hosts:
        - api.example.com
```

#### With Database

```yaml
# values-db.yaml -- reference a Secret you manage (External Secrets, Sealed
# Secrets, Vault, ...) holding jwt-secret, database-url and redis-url.
secrets:
  existingSecret: my-api-secrets
networkPolicy:
  extraEgress:
    - to:
        - namespaceSelector:
            matchLabels:
              kubernetes.io/metadata.name: databases
      ports:
        - protocol: TCP
          port: 5432
```

#### Production Configuration

```yaml
# values-production.yaml
replicaCount: 3

autoscaling:
  enabled: true
  minReplicas: 3
  maxReplicas: 20
  targetCPUUtilizationPercentage: 70

pdb:
  enabled: true
  minAvailable: 2

networkPolicy:
  enabled: true

resources:
  requests:
    cpu: 200m
    memory: 128Mi
  limits:
    cpu: 1000m
    memory: 512Mi
```

## Installation

```bash
# Install
helm install my-api ./armature -f values.yaml

# Upgrade
helm upgrade my-api ./armature -f values.yaml

# Uninstall
helm uninstall my-api
```

## Chart Structure

```
armature/
├── Chart.yaml          # Chart metadata
├── values.yaml         # Default values
├── .helmignore         # Ignore patterns
└── templates/
    ├── _helpers.tpl    # Template helpers
    ├── deployment.yaml
    ├── service.yaml
    ├── ingress.yaml
    ├── hpa.yaml
    ├── pdb.yaml
    ├── configmap.yaml
    ├── secret.yaml
    ├── serviceaccount.yaml
    ├── networkpolicy.yaml
    └── NOTES.txt       # Post-install notes
```

## Values Reference

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `3` |
| `image.repository` | Image repository (**required**) | `""` |
| `image.digest` | Image digest `sha256:...` (preferred; wins over tag) | `""` |
| `image.tag` | Image tag, used when no digest; `latest` rejected | `""` |
| `secrets.existingSecret` | Existing Secret with `jwt-secret` (+ optional `database-url`, `redis-url`) | `""` |
| `secrets.jwtSecret` | JWT secret (>= 32 chars) if the chart should create the Secret; required without `existingSecret` | `""` |
| `secrets.revision` | Bump to roll pods after rotating the Secret | `"1"` |
| `metrics.enabled` / `metrics.port` | Metrics on a dedicated port, not exposed via Service/Ingress | `true` / `9090` |
| `networkPolicy.enabled` | Default-deny NetworkPolicy with explicit allows | `true` |
| `networkPolicy.extraEgress` | Extra egress rules (databases, caches) | `[]` |
| `image.pullPolicy` | Pull policy | `IfNotPresent` |
| `service.type` | Service type | `ClusterIP` |
| `service.port` | Service port | `80` |
| `ingress.enabled` | Enable ingress | `false` |
| `autoscaling.enabled` | Enable HPA | `true` |
| `autoscaling.minReplicas` | Min replicas | `2` |
| `autoscaling.maxReplicas` | Max replicas | `10` |
| `resources.requests.cpu` | CPU request | `100m` |
| `resources.requests.memory` | Memory request | `64Mi` |
| `resources.limits.cpu` | CPU limit | `500m` |
| `resources.limits.memory` | Memory limit | `256Mi` |


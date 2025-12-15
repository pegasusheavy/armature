# Armature Helm Chart

Production-ready Helm chart for deploying Armature applications on Kubernetes.

## Quick Start

```bash
# Install from local chart
helm install my-api ./armature

# Install with custom values
helm install my-api ./armature \
  --set image.repository=myregistry/myapp \
  --set image.tag=v1.0.0 \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=api.example.com
```

## Configuration

See `armature/values.yaml` for all available options.

### Common Configurations

#### Basic Deployment

```yaml
# values-basic.yaml
replicaCount: 2
image:
  repository: ghcr.io/my-org/my-api
  tag: v1.0.0
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
# values-db.yaml
secrets:
  databaseUrl: "postgres://user:pass@host:5432/db"
  redisUrl: "redis://redis:6379"
  jwtSecret: "your-secret-key"
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
| `image.repository` | Image repository | `ghcr.io/armature-rs/armature-api` |
| `image.tag` | Image tag | Chart appVersion |
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


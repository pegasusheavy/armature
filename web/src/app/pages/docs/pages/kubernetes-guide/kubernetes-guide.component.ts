import { Component } from '@angular/core';
import { CommonModule } from '@angular/common';
import { DocPageComponent, DocPage } from '../../shared/doc-page.component';

@Component({
  selector: 'app-kubernetes-guide',
  standalone: true,
  imports: [CommonModule, DocPageComponent],
  template: `<app-doc-page [page]="page"></app-doc-page>`
})
export class KubernetesGuideComponent {
  page: DocPage = {
    title: 'Kubernetes',
    subtitle: 'Deploy and scale your Armature application on Kubernetes with production-ready configurations.',
    icon: '☸️',
    badge: 'Deployment',
    features: [
      { icon: '📦', title: 'Deployments', description: 'Rolling updates & rollbacks' },
      { icon: '🔄', title: 'Auto-scaling', description: 'HPA based on metrics' },
      { icon: '💚', title: 'Health Probes', description: 'Liveness & readiness' },
      { icon: '🔐', title: 'Secrets', description: 'Secure config management' }
    ],
    sections: [
      {
        id: 'deployment',
        title: 'Basic Deployment',
        content: `<p>Create a Kubernetes Deployment for your Armature app:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'deployment.yaml',
            code: `apiVersion: apps/v1
kind: Deployment
metadata:
  name: armature-app
  labels:
    app: armature-app
spec:
  replicas: 3
  selector:
    matchLabels:
      app: armature-app
  template:
    metadata:
      labels:
        app: armature-app
    spec:
      containers:
        - name: app
          image: myregistry/armature-app:v1.0.0
          ports:
            - containerPort: 3000
          env:
            - name: RUST_LOG
              value: "info"
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: app-secrets
                  key: database-url
          resources:
            requests:
              memory: "64Mi"
              cpu: "100m"
            limits:
              memory: "256Mi"
              cpu: "500m"`
          }
        ]
      },
      {
        id: 'health-probes',
        title: 'Health Probes',
        content: `<p>Configure liveness and readiness probes:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `spec:
  containers:
    - name: app
      # ... image, ports, env ...
      livenessProbe:
        httpGet:
          path: /health/live
          port: 3000
        initialDelaySeconds: 10
        periodSeconds: 10
        failureThreshold: 3
      readinessProbe:
        httpGet:
          path: /health/ready
          port: 3000
        initialDelaySeconds: 5
        periodSeconds: 5
        failureThreshold: 3`
          }
        ]
      },
      {
        id: 'service',
        title: 'Service',
        content: `<p>Expose your deployment with a Service:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'service.yaml',
            code: `apiVersion: v1
kind: Service
metadata:
  name: armature-app
spec:
  selector:
    app: armature-app
  ports:
    - port: 80
      targetPort: 3000
  type: ClusterIP

---
# For external access, use an Ingress
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: armature-app
  annotations:
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
spec:
  rules:
    - host: api.example.com
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: armature-app
                port:
                  number: 80
  tls:
    - hosts:
        - api.example.com
      secretName: tls-secret`
          }
        ]
      },
      {
        id: 'secrets',
        title: 'Secrets Management',
        content: `<p>Store sensitive configuration securely:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'secrets.yaml',
            code: `apiVersion: v1
kind: Secret
metadata:
  name: app-secrets
type: Opaque
stringData:
  database-url: "postgres://user:pass@db:5432/mydb"
  jwt-secret: "your-jwt-secret"
  redis-url: "redis://redis:6379"`
          },
          {
            language: 'bash',
            code: `# Create secret from command line
$ kubectl create secret generic app-secrets \\
  --from-literal=database-url="postgres://..." \\
  --from-literal=jwt-secret="..." \\
  --from-file=tls.crt --from-file=tls.key`
          }
        ]
      },
      {
        id: 'hpa',
        title: 'Horizontal Pod Autoscaler',
        content: `<p>Automatically scale based on CPU/memory usage:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'hpa.yaml',
            code: `apiVersion: autoscaling/v2
kind: HorizontalPodAutoscaler
metadata:
  name: armature-app
spec:
  scaleTargetRef:
    apiVersion: apps/v1
    kind: Deployment
    name: armature-app
  minReplicas: 2
  maxReplicas: 10
  metrics:
    - type: Resource
      resource:
        name: cpu
        target:
          type: Utilization
          averageUtilization: 70
    - type: Resource
      resource:
        name: memory
        target:
          type: Utilization
          averageUtilization: 80`
          }
        ]
      },
      {
        id: 'configmap',
        title: 'ConfigMaps',
        content: `<p>Non-sensitive configuration:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            filename: 'configmap.yaml',
            code: `apiVersion: v1
kind: ConfigMap
metadata:
  name: app-config
data:
  RUST_LOG: "info"
  MAX_CONNECTIONS: "100"
  CACHE_TTL: "3600"`
          }
        ]
      },
      {
        id: 'graceful-shutdown',
        title: 'Graceful Shutdown',
        content: `<p>Configure proper termination:</p>`,
        codeBlocks: [
          {
            language: 'bash',
            code: `spec:
  terminationGracePeriodSeconds: 60
  containers:
    - name: app
      lifecycle:
        preStop:
          exec:
            # Allow time for endpoints to update
            command: ["sleep", "5"]`
          }
        ]
      },
      {
        id: 'best-practices',
        title: 'Best Practices',
        content: `<ul>
          <li><strong>Set resource limits</strong> — Prevent runaway containers</li>
          <li><strong>Use namespaces</strong> — Isolate environments (dev, staging, prod)</li>
          <li><strong>Enable PodDisruptionBudgets</strong> — Maintain availability during updates</li>
          <li><strong>Use rolling updates</strong> — Zero-downtime deployments</li>
          <li><strong>Store secrets securely</strong> — Use external secret managers in production</li>
          <li><strong>Monitor with Prometheus</strong> — Scrape /metrics endpoint</li>
        </ul>`
      }
    ],
    relatedDocs: [
      { id: 'docker-guide', title: 'Docker', description: 'Container image creation' },
      { id: 'health-check', title: 'Health Checks', description: 'Probe endpoints' }
    ],
    seeAlso: [
      { title: 'Graceful Shutdown', id: 'graceful-shutdown' },
      { title: 'Scaling', id: 'scaling-guide' }
    ]
  };
}


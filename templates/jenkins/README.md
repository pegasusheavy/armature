# Jenkins Pipeline Templates

Production-ready Jenkins pipeline templates for Armature applications.

## Files

| File | Description |
|------|-------------|
| `Jenkinsfile` | Basic pipeline with local Rust toolchain |
| `Jenkinsfile.docker` | Pipeline using Docker agent (no local Rust needed) |
| `Jenkinsfile.multibranch` | Full CI/CD with environment deployments |

## Quick Start

1. Copy the desired Jenkinsfile to your project root:

```bash
cp templates/jenkins/Jenkinsfile ./Jenkinsfile
```

2. Create a Jenkins pipeline job pointing to your repository

3. Customize the deployment stages for your environment

## Pipeline Features

### Basic (`Jenkinsfile`)

- ✅ Build & Test
- ✅ Parallel test execution
- ✅ Format & Clippy checks
- ✅ Security audit
- ✅ Docker build
- ✅ Workspace cleanup

### Docker Agent (`Jenkinsfile.docker`)

- ✅ No local Rust installation required
- ✅ Alpine-based build container
- ✅ Cached cargo registry volume
- ✅ Static musl binary output
- ✅ Artifact archiving

### Multibranch (`Jenkinsfile.multibranch`)

- ✅ Branch-specific deployments
- ✅ develop → Development environment
- ✅ main → Staging environment
- ✅ Tags → Production (with approval)
- ✅ Helm deployment examples
- ✅ Slack notifications (commented)

## Requirements

### Basic Pipeline

- Jenkins with Pipeline plugin
- Rust toolchain installed on agent
- Docker (for Docker build stage)

### Docker Agent Pipeline

- Jenkins with Docker Pipeline plugin
- Docker installed on Jenkins host

### Multibranch Pipeline

- Jenkins Multibranch Pipeline
- Kubernetes/Helm (for deployments)
- Docker registry access

## Customization

### Change Rust Version

```groovy
agent {
    docker {
        image 'rust:1.85-alpine'  // Change version here
    }
}
```

### Add Notifications

```groovy
post {
    success {
        slackSend(color: 'good', message: "✅ Build succeeded")
    }
    failure {
        emailext(
            subject: "Build Failed: ${env.JOB_NAME}",
            body: "Check console output at ${env.BUILD_URL}",
            recipientProviders: [developers()]
        )
    }
}
```

### Add Code Coverage

```groovy
stage('Coverage') {
    steps {
        sh '''
            cargo install cargo-llvm-cov
            cargo llvm-cov --lcov --output-path lcov.info
        '''
        publishCoverage adapters: [coberturaAdapter('lcov.info')]
    }
}
```

### Add SonarQube

```groovy
stage('SonarQube') {
    steps {
        withSonarQubeEnv('SonarQube') {
            sh 'sonar-scanner'
        }
    }
}
```

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `RUST_BACKTRACE` | Enable backtraces | `1` |
| `CARGO_TERM_COLOR` | Colored output | `always` |
| `DOCKER_REGISTRY` | Docker registry URL | `your-registry.com` |

## Shared Libraries

For large organizations, consider using Jenkins Shared Libraries:

```groovy
@Library('armature-pipeline') _

armaturePipeline(
    appName: 'my-api',
    deployments: ['dev', 'staging', 'prod']
)
```


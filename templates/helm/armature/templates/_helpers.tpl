{{/*
Expand the name of the chart.
*/}}
{{- define "armature.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
*/}}
{{- define "armature.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "armature.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "armature.labels" -}}
helm.sh/chart: {{ include "armature.chart" . }}
{{ include "armature.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "armature.selectorLabels" -}}
app.kubernetes.io/name: {{ include "armature.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "armature.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "armature.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Return the image reference. Fails loudly when no image is configured: the
chart ships no default image (the project does not publish one, and a
default pointing at a registry namespace nobody controls could be claimed by
anyone). A digest wins over a tag; "latest" is refused.
*/}}
{{- define "armature.image" -}}
{{- $repo := required "image.repository is required (the chart has no default image; use your own registry)" .Values.image.repository }}
{{- if .Values.image.digest }}
{{- if not (regexMatch "^sha256:[a-f0-9]{64}$" .Values.image.digest) }}
{{- fail "image.digest must look like sha256:<64 lowercase hex chars>" }}
{{- end }}
{{- printf "%s@%s" $repo .Values.image.digest }}
{{- else }}
{{- $tag := required "set image.digest (preferred) or image.tag" .Values.image.tag }}
{{- if eq $tag "latest" }}
{{- fail "image.tag \"latest\" is not allowed; pin a version tag or, better, image.digest" }}
{{- end }}
{{- printf "%s:%s" $repo $tag }}
{{- end }}
{{- end }}

{{/*
Name of the Secret holding JWT_SECRET (and optionally DATABASE_URL /
REDIS_URL): the user's existing Secret, or the one this chart creates.
Fails when neither is configured, so JWT_SECRET can never be silently absent.
*/}}
{{- define "armature.secretName" -}}
{{- if .Values.secrets.existingSecret }}
{{- .Values.secrets.existingSecret }}
{{- else }}
{{- $jwt := required "secrets.jwtSecret is required (or set secrets.existingSecret); generate one with: openssl rand -hex 32" .Values.secrets.jwtSecret }}
{{- if lt (len $jwt) 32 }}
{{- fail "secrets.jwtSecret must be at least 32 characters; generate one with: openssl rand -hex 32" }}
{{- end }}
{{- printf "%s-secrets" (include "armature.fullname" .) }}
{{- end }}
{{- end }}


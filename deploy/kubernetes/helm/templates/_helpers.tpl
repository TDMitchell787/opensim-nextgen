{{/*
OpenSim Next Helm Chart Helper Templates
Common template definitions for consistent labeling and naming
*/}}

{{/*
Expand the name of the chart.
*/}}
{{- define "opensim-next.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "opensim-next.fullname" -}}
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
{{- define "opensim-next.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "opensim-next.labels" -}}
helm.sh/chart: {{ include "opensim-next.chart" . }}
{{ include "opensim-next.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
app.kubernetes.io/part-of: opensim-next
{{- end }}

{{/*
Selector labels
*/}}
{{- define "opensim-next.selectorLabels" -}}
app.kubernetes.io/name: {{ include "opensim-next.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "opensim-next.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "opensim-next.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the PostgreSQL service
*/}}
{{- define "opensim-next.postgresql.fullname" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "%s-%s" .Release.Name "postgresql" | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- .Values.postgresql.external.host }}
{{- end }}
{{- end }}

{{/*
Create the name of the Redis service
*/}}
{{- define "opensim-next.redis.fullname" -}}
{{- if .Values.redis.enabled }}
{{- printf "%s-%s" .Release.Name "redis-master" | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- .Values.redis.external.host }}
{{- end }}
{{- end }}

{{/*
Create the PostgreSQL database URL
*/}}
{{- define "opensim-next.postgresql.url" -}}
{{- if .Values.postgresql.enabled }}
{{- printf "postgresql://%s:%s@%s:%d/%s" .Values.postgresql.auth.username .Values.postgresql.auth.password (include "opensim-next.postgresql.fullname" .) 5432 .Values.postgresql.auth.database }}
{{- else }}
{{- .Values.postgresql.external.url }}
{{- end }}
{{- end }}

{{/*
Create the Redis URL
*/}}
{{- define "opensim-next.redis.url" -}}
{{- if .Values.redis.enabled }}
{{- if .Values.redis.auth.enabled }}
{{- printf "redis://:%s@%s:%d" .Values.redis.auth.password (include "opensim-next.redis.fullname" .) 6379 }}
{{- else }}
{{- printf "redis://%s:%d" (include "opensim-next.redis.fullname" .) 6379 }}
{{- end }}
{{- else }}
{{- .Values.redis.external.url }}
{{- end }}
{{- end }}

{{/*
Create environment variables for OpenSim Next
*/}}
{{- define "opensim-next.environment" -}}
- name: DATABASE_URL
  value: {{ include "opensim-next.postgresql.url" . | quote }}
- name: REDIS_URL
  value: {{ include "opensim-next.redis.url" . | quote }}
- name: OPENSIM_INSTANCE_ID
  value: {{ printf "%s-%s" .Release.Name .Release.Namespace | quote }}
- name: KUBERNETES_NAMESPACE
  valueFrom:
    fieldRef:
      fieldPath: metadata.namespace
- name: KUBERNETES_POD_NAME
  valueFrom:
    fieldRef:
      fieldPath: metadata.name
- name: KUBERNETES_POD_IP
  valueFrom:
    fieldRef:
      fieldPath: status.podIP
- name: KUBERNETES_NODE_NAME
  valueFrom:
    fieldRef:
      fieldPath: spec.nodeName
{{- range $key, $value := .Values.opensim.env }}
- name: {{ $key }}
  value: {{ $value | quote }}
{{- end }}
{{- end }}

{{/*
Create image pull policy
*/}}
{{- define "opensim-next.imagePullPolicy" -}}
{{- if .Values.global.imageRegistry }}
{{- .Values.opensim.image.pullPolicy | default "IfNotPresent" }}
{{- else }}
{{- .Values.opensim.image.pullPolicy | default "IfNotPresent" }}
{{- end }}
{{- end }}

{{/*
Create image name
*/}}
{{- define "opensim-next.image" -}}
{{- $registry := .Values.opensim.image.registry | default .Values.global.imageRegistry }}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry .Values.opensim.image.repository (.Values.opensim.image.tag | default .Chart.AppVersion) }}
{{- else }}
{{- printf "%s:%s" .Values.opensim.image.repository (.Values.opensim.image.tag | default .Chart.AppVersion) }}
{{- end }}
{{- end }}

{{/*
Create storage class name
*/}}
{{- define "opensim-next.storageClass" -}}
{{- if .Values.global.storageClass }}
{{- .Values.global.storageClass }}
{{- else if .storageClass }}
{{- .storageClass }}
{{- else }}
{{- "" }}
{{- end }}
{{- end }}

{{/*
Create network policy name
*/}}
{{- define "opensim-next.networkPolicy.name" -}}
{{- printf "%s-network-policy" (include "opensim-next.fullname" .) }}
{{- end }}

{{/*
Create pod security policy name
*/}}
{{- define "opensim-next.podSecurityPolicy.name" -}}
{{- printf "%s-psp" (include "opensim-next.fullname" .) }}
{{- end }}

{{/*
Create service monitor name
*/}}
{{- define "opensim-next.serviceMonitor.name" -}}
{{- printf "%s-metrics" (include "opensim-next.fullname" .) }}
{{- end }}

{{/*
Validate required values
*/}}
{{- define "opensim-next.validateValues" -}}
{{- if and (not .Values.postgresql.enabled) (not .Values.postgresql.external.url) }}
{{- fail "PostgreSQL must be enabled or external URL must be provided" }}
{{- end }}
{{- if and .Values.opensim.autoscaling.enabled (lt (int .Values.opensim.autoscaling.minReplicas) 1) }}
{{- fail "Autoscaling minReplicas must be at least 1" }}
{{- end }}
{{- if and .Values.opensim.autoscaling.enabled (gt (int .Values.opensim.autoscaling.minReplicas) (int .Values.opensim.autoscaling.maxReplicas)) }}
{{- fail "Autoscaling minReplicas cannot be greater than maxReplicas" }}
{{- end }}
{{- end }}

{{/*
Create TLS secret name for ingress
*/}}
{{- define "opensim-next.ingress.tlsSecretName" -}}
{{- if .Values.opensim.ingress.tls }}
{{- range .Values.opensim.ingress.tls }}
{{- .secretName }}
{{- end }}
{{- else }}
{{- printf "%s-tls" (include "opensim-next.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Create monitoring labels
*/}}
{{- define "opensim-next.monitoring.labels" -}}
monitoring: "true"
metrics.prometheus.io/scrape: "true"
metrics.prometheus.io/port: "9100"
metrics.prometheus.io/path: "/metrics"
{{- end }}

{{/*
Create security annotations
*/}}
{{- define "opensim-next.security.annotations" -}}
seccomp.security.alpha.kubernetes.io/pod: runtime/default
container.apparmor.security.beta.kubernetes.io/opensim-next: runtime/default
{{- end }}

{{/*
Create resource limits based on environment
*/}}
{{- define "opensim-next.resources" -}}
{{- if .Values.development.enabled }}
{{- if .Values.development.disableResourceLimits }}
{{- /* No resource limits in development mode */ -}}
{{- else }}
requests:
  cpu: 100m
  memory: 256Mi
limits:
  cpu: 500m
  memory: 1Gi
{{- end }}
{{- else }}
{{- with .Values.opensim.resources }}
{{- toYaml . }}
{{- end }}
{{- end }}
{{- end }}
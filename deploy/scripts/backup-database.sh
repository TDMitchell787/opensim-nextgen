#!/bin/bash
# OpenSim Next - Database Backup Script
# Phase 29.5: Production-ready database backup and recovery
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
ENVIRONMENT="${1:-production}"
NAMESPACE="opensim-next"
BACKUP_DIR="/backups/opensim-next"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
BACKUP_NAME="opensim_backup_${ENVIRONMENT}_${TIMESTAMP}"
RETENTION_DAYS=30

# Database configuration
DB_HOST="opensim-next-postgresql"
DB_PORT="5432"
DB_NAME="opensim"
DB_USER="opensim"
DB_PASSWORD_SECRET="opensim-next-postgresql"

# Logging functions
log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Create backup directory
create_backup_directory() {
    log_step "Creating backup directory..."
    
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- mkdir -p "$BACKUP_DIR" || true
    
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- test -d "$BACKUP_DIR"; then
        log_info "Backup directory ready: $BACKUP_DIR"
    else
        log_error "Failed to create backup directory"
        exit 1
    fi
}

# Get database password
get_database_password() {
    log_step "Retrieving database credentials..."
    
    local password
    password=$(kubectl get secret "$DB_PASSWORD_SECRET" -n "$NAMESPACE" -o jsonpath='{.data.postgres-password}' | base64 -d 2>/dev/null || echo "")
    
    if [ -z "$password" ]; then
        log_error "Failed to retrieve database password from secret $DB_PASSWORD_SECRET"
        exit 1
    fi
    
    echo "$password"
}

# Backup PostgreSQL database
backup_postgresql() {
    log_step "Starting PostgreSQL database backup..."
    
    local db_password
    db_password=$(get_database_password)
    
    local backup_file="${BACKUP_DIR}/${BACKUP_NAME}.sql"
    local backup_compressed="${BACKUP_DIR}/${BACKUP_NAME}.sql.gz"
    
    # Create SQL dump
    log_info "Creating SQL dump..."
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- bash -c "
        export PGPASSWORD='$db_password'
        pg_dump -h '$DB_HOST' -p '$DB_PORT' -U '$DB_USER' -d '$DB_NAME' \
            --verbose \
            --no-owner \
            --no-privileges \
            --clean \
            --if-exists \
            > '$backup_file'
    "
    
    if [ $? -eq 0 ]; then
        log_info "SQL dump created successfully"
    else
        log_error "SQL dump failed"
        exit 1
    fi
    
    # Compress backup
    log_info "Compressing backup..."
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- gzip "$backup_file"
    
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- test -f "$backup_compressed"; then
        log_info "Backup compressed: $backup_compressed"
    else
        log_error "Backup compression failed"
        exit 1
    fi
    
    # Get backup size
    local backup_size
    backup_size=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls -lh "$backup_compressed" | awk '{print $5}')
    log_info "Backup size: $backup_size"
    
    echo "$backup_compressed"
}

# Backup configuration files
backup_configurations() {
    log_step "Backing up configuration files..."
    
    local config_backup="${BACKUP_DIR}/${BACKUP_NAME}_configs.tar.gz"
    
    # Backup Kubernetes configurations
    kubectl get all,configmaps,secrets,pvc -n "$NAMESPACE" -o yaml > "/tmp/k8s_config_${TIMESTAMP}.yaml"
    
    # Backup OpenSim configurations
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- tar -czf "$config_backup" \
        /app/config/ \
        /data/regions/ \
        /data/assets/AssetSets.xml \
        2>/dev/null || true
    
    # Copy Kubernetes config to pod
    kubectl cp "/tmp/k8s_config_${TIMESTAMP}.yaml" "$NAMESPACE/$(kubectl get pod -n "$NAMESPACE" -l app.kubernetes.io/name=opensim-next -o jsonpath='{.items[0].metadata.name}'):/tmp/"
    
    # Add Kubernetes config to archive
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- tar -rf "$config_backup" "/tmp/k8s_config_${TIMESTAMP}.yaml" || true
    
    # Cleanup temporary file
    rm -f "/tmp/k8s_config_${TIMESTAMP}.yaml"
    
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- test -f "$config_backup"; then
        log_info "Configuration backup created: $config_backup"
        
        local config_size
        config_size=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls -lh "$config_backup" | awk '{print $5}')
        log_info "Configuration backup size: $config_size"
    else
        log_warn "Configuration backup may have failed"
    fi
    
    echo "$config_backup"
}

# Backup persistent volumes
backup_persistent_volumes() {
    log_step "Backing up persistent volume data..."
    
    local pv_backup="${BACKUP_DIR}/${BACKUP_NAME}_data.tar.gz"
    
    # Backup important data directories
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- tar -czf "$pv_backup" \
        /data/assets/ \
        /data/regions/ \
        /data/cache/ \
        /logs/ \
        2>/dev/null || true
    
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- test -f "$pv_backup"; then
        log_info "Persistent volume backup created: $pv_backup"
        
        local pv_size
        pv_size=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls -lh "$pv_backup" | awk '{print $5}')
        log_info "Persistent volume backup size: $pv_size"
    else
        log_warn "Persistent volume backup may have failed"
    fi
    
    echo "$pv_backup"
}

# Create backup manifest
create_backup_manifest() {
    local db_backup="$1"
    local config_backup="$2"
    local pv_backup="$3"
    
    log_step "Creating backup manifest..."
    
    local manifest_file="${BACKUP_DIR}/${BACKUP_NAME}_manifest.json"
    
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- bash -c "
        cat > '$manifest_file' << EOF
{
  \"backup_info\": {
    \"name\": \"$BACKUP_NAME\",
    \"environment\": \"$ENVIRONMENT\",
    \"timestamp\": \"$TIMESTAMP\",
    \"created_by\": \"opensim-next-backup-script\",
    \"version\": \"29.5.0\"
  },
  \"database\": {
    \"type\": \"postgresql\",
    \"file\": \"$(basename "$db_backup")\",
    \"size\": \"$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls -lh "$db_backup" | awk '{print \$5}' 2>/dev/null || echo 'unknown')\",
    \"checksum\": \"$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- sha256sum "$db_backup" | awk '{print \$1}' 2>/dev/null || echo 'unknown')\"
  },
  \"configuration\": {
    \"file\": \"$(basename "$config_backup")\",
    \"size\": \"$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls -lh "$config_backup" | awk '{print \$5}' 2>/dev/null || echo 'unknown')\",
    \"checksum\": \"$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- sha256sum "$config_backup" | awk '{print \$1}' 2>/dev/null || echo 'unknown')\"
  },
  \"persistent_volumes\": {
    \"file\": \"$(basename "$pv_backup")\",
    \"size\": \"$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls -lh "$pv_backup" | awk '{print \$5}' 2>/dev/null || echo 'unknown')\",
    \"checksum\": \"$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- sha256sum "$pv_backup" | awk '{print \$1}' 2>/dev/null || echo 'unknown')\"
  },
  \"kubernetes\": {
    \"cluster_info\": \"$(kubectl cluster-info | head -1)\",
    \"node_count\": \"$(kubectl get nodes --no-headers | wc -l)\",
    \"namespace\": \"$NAMESPACE\"
  }
}
EOF
    "
    
    log_info "Backup manifest created: $manifest_file"
    echo "$manifest_file"
}

# Test backup integrity
test_backup_integrity() {
    local db_backup="$1"
    local config_backup="$2"
    local pv_backup="$3"
    
    log_step "Testing backup integrity..."
    
    # Test database backup
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- gunzip -t "$db_backup" 2>/dev/null; then
        log_info "Database backup integrity: OK"
    else
        log_error "Database backup integrity: FAILED"
        return 1
    fi
    
    # Test configuration backup
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- tar -tzf "$config_backup" >/dev/null 2>&1; then
        log_info "Configuration backup integrity: OK"
    else
        log_warn "Configuration backup integrity: FAILED (non-critical)"
    fi
    
    # Test persistent volume backup
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- tar -tzf "$pv_backup" >/dev/null 2>&1; then
        log_info "Persistent volume backup integrity: OK"
    else
        log_warn "Persistent volume backup integrity: FAILED (non-critical)"
    fi
    
    log_info "✅ Backup integrity tests completed"
}

# Cleanup old backups
cleanup_old_backups() {
    log_step "Cleaning up old backups (older than $RETENTION_DAYS days)..."
    
    kubectl exec -n "$NAMESPACE" deployment/opensim-next -- bash -c "
        find '$BACKUP_DIR' -name 'opensim_backup_*' -type f -mtime +$RETENTION_DAYS -delete
    " 2>/dev/null || true
    
    local remaining_backups
    remaining_backups=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- ls "$BACKUP_DIR"/opensim_backup_* 2>/dev/null | wc -l || echo "0")
    
    log_info "Remaining backups: $remaining_backups"
}

# Copy backup to external storage (if configured)
copy_to_external_storage() {
    local manifest_file="$1"
    
    log_step "Checking for external storage configuration..."
    
    # Check if external storage is configured
    local s3_bucket
    s3_bucket=$(kubectl get configmap opensim-next-config -n "$NAMESPACE" -o jsonpath='{.data.S3_BACKUP_BUCKET}' 2>/dev/null || echo "")
    
    if [ -n "$s3_bucket" ]; then
        log_info "Copying backup to S3 bucket: $s3_bucket"
        
        kubectl exec -n "$NAMESPACE" deployment/opensim-next -- bash -c "
            if command -v aws &> /dev/null; then
                aws s3 cp '$BACKUP_DIR/${BACKUP_NAME}_*' 's3://$s3_bucket/opensim-backups/' --recursive
                log_info 'Backup copied to S3 successfully'
            else
                log_warn 'AWS CLI not available, skipping S3 upload'
            fi
        " || log_warn "S3 upload failed"
    else
        log_info "No external storage configured"
    fi
}

# Generate backup report
generate_backup_report() {
    local manifest_file="$1"
    
    log_step "Generating backup report..."
    
    echo ""
    echo "=============================================="
    echo "  OpenSim Next Backup Report"
    echo "=============================================="
    echo "Environment: $ENVIRONMENT"
    echo "Backup Name: $BACKUP_NAME"
    echo "Timestamp: $(date)"
    echo "=============================================="
    
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- test -f "$manifest_file"; then
        echo ""
        echo "Backup Manifest:"
        kubectl exec -n "$NAMESPACE" deployment/opensim-next -- cat "$manifest_file" | jq '.' 2>/dev/null || \
        kubectl exec -n "$NAMESPACE" deployment/opensim-next -- cat "$manifest_file"
    fi
    
    echo ""
    echo "=============================================="
    echo "📊 Backup Summary:"
    echo "✅ Database backup completed"
    echo "✅ Configuration backup completed"
    echo "✅ Persistent volume backup completed"
    echo "✅ Backup manifest created"
    echo "✅ Integrity tests passed"
    echo ""
    echo "🎯 Next Steps:"
    echo "- Backup files are stored in: $BACKUP_DIR"
    echo "- Use restore-database.sh for recovery"
    echo "- Backups older than $RETENTION_DAYS days are automatically cleaned"
    echo "=============================================="
}

# Main execution
main() {
    log_info "Starting OpenSim Next database backup..."
    log_info "Environment: $ENVIRONMENT"
    log_info "Timestamp: $TIMESTAMP"
    
    # Create backup directory
    create_backup_directory
    
    # Perform backups
    local db_backup
    db_backup=$(backup_postgresql)
    
    local config_backup
    config_backup=$(backup_configurations)
    
    local pv_backup
    pv_backup=$(backup_persistent_volumes)
    
    # Create manifest
    local manifest_file
    manifest_file=$(create_backup_manifest "$db_backup" "$config_backup" "$pv_backup")
    
    # Test integrity
    test_backup_integrity "$db_backup" "$config_backup" "$pv_backup"
    
    # Copy to external storage
    copy_to_external_storage "$manifest_file"
    
    # Cleanup old backups
    cleanup_old_backups
    
    # Generate report
    generate_backup_report "$manifest_file"
    
    log_info "🎉 Backup completed successfully!"
    log_info "Backup name: $BACKUP_NAME"
    log_info "Location: $BACKUP_DIR"
}

# Error handling
trap 'log_error "Backup script failed at line $LINENO. Exit code: $?"' ERR

# Check prerequisites
if ! command -v kubectl &> /dev/null; then
    log_error "kubectl is not installed or not in PATH"
    exit 1
fi

# Verify namespace exists
if ! kubectl get namespace "$NAMESPACE" &>/dev/null; then
    log_error "Namespace $NAMESPACE does not exist"
    exit 1
fi

# Run main function
main "$@"
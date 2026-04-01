#!/bin/bash
# OpenSim Next - Production Deployment Validation Script
# Phase 29.5: Comprehensive deployment validation and health checks
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
ENVIRONMENT="${1:-staging}"
NAMESPACE="${2:-opensim-next-${ENVIRONMENT}}"
TIMEOUT=300
MAX_RETRIES=10
RETRY_DELAY=30

# Validation results
VALIDATION_RESULTS=()
FAILED_CHECKS=0
TOTAL_CHECKS=0

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

# Validation functions
check_result() {
    local test_name="$1"
    local result="$2"
    local details="${3:-}"
    
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    
    if [ "$result" -eq 0 ]; then
        VALIDATION_RESULTS+=("✅ $test_name: PASSED")
        log_info "$test_name: PASSED"
    else
        VALIDATION_RESULTS+=("❌ $test_name: FAILED - $details")
        log_error "$test_name: FAILED - $details"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
}

# Kubernetes resource validation
validate_kubernetes_resources() {
    log_step "Validating Kubernetes resources..."
    
    # Check namespace exists
    if kubectl get namespace "$NAMESPACE" &>/dev/null; then
        check_result "Namespace existence" 0
    else
        check_result "Namespace existence" 1 "Namespace $NAMESPACE not found"
        return 1
    fi
    
    # Check deployment status
    local deployment_ready
    deployment_ready=$(kubectl get deployment opensim-next -n "$NAMESPACE" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
    local deployment_desired
    deployment_desired=$(kubectl get deployment opensim-next -n "$NAMESPACE" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "1")
    
    if [ "$deployment_ready" -eq "$deployment_desired" ] && [ "$deployment_ready" -gt 0 ]; then
        check_result "Deployment readiness" 0
    else
        check_result "Deployment readiness" 1 "Ready: $deployment_ready, Desired: $deployment_desired"
    fi
    
    # Check pod status
    local pod_status
    pod_status=$(kubectl get pods -n "$NAMESPACE" -l app.kubernetes.io/name=opensim-next -o jsonpath='{.items[*].status.phase}' 2>/dev/null || echo "")
    
    if echo "$pod_status" | grep -q "Running"; then
        check_result "Pod status" 0
    else
        check_result "Pod status" 1 "Pods not running: $pod_status"
    fi
    
    # Check service endpoints
    local endpoints
    endpoints=$(kubectl get endpoints opensim-next -n "$NAMESPACE" -o jsonpath='{.subsets[*].addresses[*].ip}' 2>/dev/null || echo "")
    
    if [ -n "$endpoints" ]; then
        check_result "Service endpoints" 0
    else
        check_result "Service endpoints" 1 "No service endpoints found"
    fi
    
    # Check persistent volume claims
    local pvc_status
    pvc_status=$(kubectl get pvc -n "$NAMESPACE" -o jsonpath='{.items[*].status.phase}' 2>/dev/null || echo "")
    
    if echo "$pvc_status" | grep -qv "Pending\|Failed" || [ -z "$pvc_status" ]; then
        check_result "Persistent volumes" 0
    else
        check_result "Persistent volumes" 1 "PVC issues: $pvc_status"
    fi
}

# Service connectivity validation
validate_service_connectivity() {
    log_step "Validating service connectivity..."
    
    # Get service IP and ports
    local service_ip
    service_ip=$(kubectl get service opensim-next -n "$NAMESPACE" -o jsonpath='{.spec.clusterIP}' 2>/dev/null || echo "")
    
    if [ -z "$service_ip" ]; then
        check_result "Service IP resolution" 1 "Could not resolve service IP"
        return 1
    fi
    
    # Test web interface (port 8080)
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s --max-time 10 "http://${service_ip}:8080/health" &>/dev/null; then
        check_result "Web interface connectivity" 0
    else
        check_result "Web interface connectivity" 1 "Cannot reach web interface on port 8080"
    fi
    
    # Test WebSocket server (port 9001)
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- nc -z "$service_ip" 9001 &>/dev/null; then
        check_result "WebSocket server connectivity" 0
    else
        check_result "WebSocket server connectivity" 1 "Cannot reach WebSocket server on port 9001"
    fi
    
    # Test Second Life protocol (port 9000)
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- nc -z "$service_ip" 9000 &>/dev/null; then
        check_result "Second Life protocol connectivity" 0
    else
        check_result "Second Life protocol connectivity" 1 "Cannot reach SL protocol on port 9000"
    fi
    
    # Test metrics endpoint (port 9100)
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s --max-time 5 "http://${service_ip}:9100/metrics" | grep -q "opensim_" &>/dev/null; then
        check_result "Metrics endpoint" 0
    else
        check_result "Metrics endpoint" 1 "Metrics endpoint not responding properly"
    fi
    
    # Test admin API (port 9200)
    if kubectl exec -n "$NAMESPACE" deployment/opensim-next -- nc -z "$service_ip" 9200 &>/dev/null; then
        check_result "Admin API connectivity" 0
    else
        check_result "Admin API connectivity" 1 "Cannot reach admin API on port 9200"
    fi
}

# Database connectivity validation
validate_database_connectivity() {
    log_step "Validating database connectivity..."
    
    # Test PostgreSQL connection
    local db_test_result
    db_test_result=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- /app/tools/db-health-check.sh 2>/dev/null || echo "FAILED")
    
    if echo "$db_test_result" | grep -q "SUCCESS"; then
        check_result "Database connectivity" 0
    else
        check_result "Database connectivity" 1 "Database connection test failed"
    fi
    
    # Test Redis connection (if enabled)
    local redis_test_result
    redis_test_result=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- redis-cli -h opensim-next-redis ping 2>/dev/null || echo "FAILED")
    
    if echo "$redis_test_result" | grep -q "PONG"; then
        check_result "Redis connectivity" 0
    else
        check_result "Redis connectivity" 1 "Redis connection test failed"
    fi
}

# Application health validation
validate_application_health() {
    log_step "Validating application health..."
    
    # Wait for application to fully start
    local retry_count=0
    while [ $retry_count -lt $MAX_RETRIES ]; do
        local health_status
        health_status=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s --max-time 10 "http://localhost:9100/health" 2>/dev/null || echo "FAILED")
        
        if echo "$health_status" | grep -q '"status":"healthy"'; then
            check_result "Application health check" 0
            break
        else
            log_warn "Health check attempt $((retry_count + 1))/$MAX_RETRIES failed, retrying in ${RETRY_DELAY}s..."
            sleep $RETRY_DELAY
            retry_count=$((retry_count + 1))
        fi
    done
    
    if [ $retry_count -eq $MAX_RETRIES ]; then
        check_result "Application health check" 1 "Health check failed after $MAX_RETRIES attempts"
    fi
    
    # Test avatar system
    local avatar_test
    avatar_test=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s --max-time 10 "http://localhost:9100/api/avatars/health" 2>/dev/null || echo "FAILED")
    
    if echo "$avatar_test" | grep -q "ok"; then
        check_result "Avatar system health" 0
    else
        check_result "Avatar system health" 1 "Avatar system health check failed"
    fi
    
    # Test social features
    local social_test
    social_test=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s --max-time 10 "http://localhost:9100/api/social/health" 2>/dev/null || echo "FAILED")
    
    if echo "$social_test" | grep -q "ok"; then
        check_result "Social features health" 0
    else
        check_result "Social features health" 1 "Social features health check failed"
    fi
    
    # Test economy system
    local economy_test
    economy_test=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s --max-time 10 "http://localhost:9100/api/economy/health" 2>/dev/null || echo "FAILED")
    
    if echo "$economy_test" | grep -q "ok"; then
        check_result "Economy system health" 0
    else
        check_result "Economy system health" 1 "Economy system health check failed"
    fi
}

# Performance validation
validate_performance() {
    log_step "Validating performance metrics..."
    
    # Check resource usage
    local cpu_usage
    cpu_usage=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s "http://localhost:9100/metrics" | grep "cpu_usage_percent" | awk '{print $2}' | head -1)
    
    if [ -n "$cpu_usage" ] && [ "$(echo "$cpu_usage < 80" | bc -l 2>/dev/null || echo 0)" -eq 1 ]; then
        check_result "CPU usage check" 0
    else
        check_result "CPU usage check" 1 "CPU usage too high or not available: $cpu_usage%"
    fi
    
    # Check memory usage
    local memory_usage
    memory_usage=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -s "http://localhost:9100/metrics" | grep "memory_usage_percent" | awk '{print $2}' | head -1)
    
    if [ -n "$memory_usage" ] && [ "$(echo "$memory_usage < 85" | bc -l 2>/dev/null || echo 0)" -eq 1 ]; then
        check_result "Memory usage check" 0
    else
        check_result "Memory usage check" 1 "Memory usage too high or not available: $memory_usage%"
    fi
    
    # Check response times
    local response_time
    response_time=$(kubectl exec -n "$NAMESPACE" deployment/opensim-next -- curl -w "%{time_total}" -s --max-time 10 "http://localhost:8080/health" -o /dev/null 2>/dev/null || echo "999")
    
    if [ "$(echo "$response_time < 2.0" | bc -l 2>/dev/null || echo 0)" -eq 1 ]; then
        check_result "Response time check" 0
    else
        check_result "Response time check" 1 "Response time too slow: ${response_time}s"
    fi
}

# Security validation
validate_security() {
    log_step "Validating security configuration..."
    
    # Check pod security context
    local security_context
    security_context=$(kubectl get pod -n "$NAMESPACE" -l app.kubernetes.io/name=opensim-next -o jsonpath='{.items[0].spec.securityContext.runAsNonRoot}' 2>/dev/null || echo "false")
    
    if [ "$security_context" = "true" ]; then
        check_result "Pod security context" 0
    else
        check_result "Pod security context" 1 "Pod not running as non-root user"
    fi
    
    # Check network policies
    local network_policies
    network_policies=$(kubectl get networkpolicy -n "$NAMESPACE" -o name 2>/dev/null | wc -l)
    
    if [ "$network_policies" -gt 0 ]; then
        check_result "Network policies" 0
    else
        check_result "Network policies" 1 "No network policies found"
    fi
    
    # Check TLS configuration
    local tls_config
    tls_config=$(kubectl get ingress -n "$NAMESPACE" -o jsonpath='{.items[*].spec.tls}' 2>/dev/null || echo "")
    
    if [ -n "$tls_config" ] || [ "$ENVIRONMENT" = "development" ]; then
        check_result "TLS configuration" 0
    else
        check_result "TLS configuration" 1 "TLS not configured for production environment"
    fi
}

# Generate validation report
generate_report() {
    log_step "Generating validation report..."
    
    echo ""
    echo "=============================================="
    echo "  OpenSim Next Deployment Validation Report"
    echo "=============================================="
    echo "Environment: $ENVIRONMENT"
    echo "Namespace: $NAMESPACE"
    echo "Timestamp: $(date)"
    echo "=============================================="
    echo ""
    
    for result in "${VALIDATION_RESULTS[@]}"; do
        echo "$result"
    done
    
    echo ""
    echo "=============================================="
    echo "Summary: $((TOTAL_CHECKS - FAILED_CHECKS))/$TOTAL_CHECKS checks passed"
    
    if [ $FAILED_CHECKS -eq 0 ]; then
        echo -e "${GREEN}🎉 All validation checks PASSED!${NC}"
        echo -e "${GREEN}✅ OpenSim Next deployment is healthy and ready${NC}"
        return 0
    else
        echo -e "${RED}❌ $FAILED_CHECKS validation checks FAILED${NC}"
        echo -e "${RED}🚨 Deployment requires attention${NC}"
        return 1
    fi
}

# Main execution
main() {
    log_info "Starting OpenSim Next deployment validation..."
    log_info "Environment: $ENVIRONMENT, Namespace: $NAMESPACE"
    
    # Run all validations
    validate_kubernetes_resources
    validate_service_connectivity
    validate_database_connectivity
    validate_application_health
    validate_performance
    validate_security
    
    # Generate and display report
    generate_report
    
    # Exit with appropriate code
    if [ $FAILED_CHECKS -eq 0 ]; then
        log_info "✅ Deployment validation completed successfully!"
        exit 0
    else
        log_error "❌ Deployment validation failed with $FAILED_CHECKS issues"
        exit 1
    fi
}

# Error handling
trap 'log_error "Validation script failed at line $LINENO. Exit code: $?"' ERR

# Check prerequisites
if ! command -v kubectl &> /dev/null; then
    log_error "kubectl is not installed or not in PATH"
    exit 1
fi

if ! command -v bc &> /dev/null; then
    log_warn "bc is not installed, some numeric comparisons may fail"
fi

# Run main function
main "$@"
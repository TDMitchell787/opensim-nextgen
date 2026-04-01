#!/bin/bash
# OpenSim Next - Portainer + Docker Desktop Setup Script
# Phase 29.3: Complete Portainer deployment with Docker Desktop integration
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NAMESPACE="portainer"
OPENSIM_NAMESPACE="opensim-next"
TIMEOUT="300s"

# Logo and header
echo -e "${BLUE}"
cat << 'EOF'
 ____                   ____  _            _   _           _   
/ __ \                 / __ \(_)          | \ | |         | |  
| |  | |_ __   ___ _ __ | |  | |_ _ __ ___  |  \| | _____  _| |_ 
| |  | | '_ \ / _ \ '_ \| |  | | | '_ ` _ \ | . ` |/ _ \ \/ / __|
| |__| | |_) |  __/ | | | |__| | | | | | | | |\  |  __/>  <| |_ 
 \____/| .__/ \___|_| |_|\____/|_|_| |_| |_|_| \_|\___/_/\_\\__|
       | |                                                     
       |_|                                                     

OpenSim Next - Portainer + Docker Desktop Setup
Phase 29.3: Complete Production Deployment Interface
EOF
echo -e "${NC}"

# Function to print status messages
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Function to check prerequisites
check_prerequisites() {
    print_step "Checking prerequisites..."
    
    # Check if kubectl is installed
    if ! command -v kubectl &> /dev/null; then
        print_error "kubectl is not installed. Please install kubectl first."
        exit 1
    fi
    
    # Check if helm is installed
    if ! command -v helm &> /dev/null; then
        print_error "Helm is not installed. Please install Helm 3.8+ first."
        exit 1
    fi
    
    # Check if Docker Desktop is running
    if ! kubectl config current-context | grep -q "docker-desktop"; then
        print_error "Docker Desktop Kubernetes is not the current context."
        print_error "Please enable Kubernetes in Docker Desktop and set context:"
        print_error "kubectl config use-context docker-desktop"
        exit 1
    fi
    
    # Check if cluster is accessible
    if ! kubectl cluster-info &> /dev/null; then
        print_error "Cannot access Kubernetes cluster. Please check Docker Desktop."
        exit 1
    fi
    
    print_status "Prerequisites check passed ✓"
}

# Function to install NGINX Ingress Controller
install_ingress_controller() {
    print_step "Installing NGINX Ingress Controller for Docker Desktop..."
    
    # Check if ingress-nginx namespace exists
    if kubectl get namespace ingress-nginx &> /dev/null; then
        print_status "NGINX Ingress Controller already exists"
    else
        print_status "Installing NGINX Ingress Controller..."
        kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/controller-v1.8.2/deploy/static/provider/cloud/deploy.yaml
        
        print_status "Waiting for ingress controller to be ready..."
        kubectl wait --namespace ingress-nginx \
            --for=condition=ready pod \
            --selector=app.kubernetes.io/component=controller \
            --timeout=${TIMEOUT}
    fi
    
    print_status "NGINX Ingress Controller ready ✓"
}

# Function to deploy Portainer
deploy_portainer() {
    print_step "Deploying Portainer to Kubernetes..."
    
    # Apply Portainer deployment
    kubectl apply -f "${SCRIPT_DIR}/portainer-deployment.yaml"
    
    print_status "Waiting for Portainer to be ready..."
    kubectl wait --for=condition=ready pod -l app.kubernetes.io/name=portainer -n ${NAMESPACE} --timeout=${TIMEOUT}
    
    print_status "Portainer deployment completed ✓"
}

# Function to setup port forwarding
setup_port_forwarding() {
    print_step "Setting up port forwarding for local access..."
    
    # Check if port forwarding is already running
    if pgrep -f "kubectl port-forward.*portainer.*9000:9000" > /dev/null; then
        print_status "Portainer port forwarding already active"
    else
        print_status "Starting Portainer port forwarding in background..."
        nohup kubectl port-forward -n ${NAMESPACE} svc/portainer 9000:9000 > /dev/null 2>&1 &
        sleep 2
        
        if pgrep -f "kubectl port-forward.*portainer.*9000:9000" > /dev/null; then
            print_status "Port forwarding started successfully ✓"
        else
            print_warning "Port forwarding may have failed. Try manually: kubectl port-forward -n ${NAMESPACE} svc/portainer 9000:9000"
        fi
    fi
}

# Function to create OpenSim Next namespace
create_opensim_namespace() {
    print_step "Creating OpenSim Next namespace..."
    
    if kubectl get namespace ${OPENSIM_NAMESPACE} &> /dev/null; then
        print_status "OpenSim Next namespace already exists"
    else
        kubectl create namespace ${OPENSIM_NAMESPACE}
        kubectl label namespace ${OPENSIM_NAMESPACE} app.kubernetes.io/name=opensim-next
        print_status "OpenSim Next namespace created ✓"
    fi
}

# Function to setup Helm repositories
setup_helm_repos() {
    print_step "Setting up Helm repositories..."
    
    print_status "Adding required Helm repositories..."
    helm repo add bitnami https://charts.bitnami.com/bitnami
    helm repo add prometheus-community https://prometheus-community.github.io/helm-charts
    helm repo add grafana https://grafana.github.io/helm-charts
    helm repo update
    
    print_status "Helm repositories configured ✓"
}

# Function to display access information
display_access_info() {
    print_step "Deployment completed! Access information:"
    
    echo
    echo -e "${GREEN}🎉 Portainer Management Interface:${NC}"
    echo -e "   ${BLUE}URL:${NC} http://localhost:9000"
    echo -e "   ${BLUE}Initial Setup:${NC} Create admin user on first access"
    echo
    
    echo -e "${GREEN}📋 Quick Commands:${NC}"
    echo -e "   ${BLUE}Access Portainer:${NC} kubectl port-forward -n ${NAMESPACE} svc/portainer 9000:9000"
    echo -e "   ${BLUE}Check Status:${NC} kubectl get pods -n ${NAMESPACE}"
    echo -e "   ${BLUE}View Logs:${NC} kubectl logs -n ${NAMESPACE} deployment/portainer"
    echo
    
    echo -e "${GREEN}🚀 Next Steps:${NC}"
    echo -e "   1. Open http://localhost:9000 in your browser"
    echo -e "   2. Create Portainer admin user"
    echo -e "   3. Select 'Get Started' for local Kubernetes environment"
    echo -e "   4. Deploy OpenSim Next using Helm chart in Portainer"
    echo
    
    echo -e "${GREEN}📖 Documentation:${NC}"
    echo -e "   ${BLUE}Setup Guide:${NC} kubectl get configmap docker-desktop-setup-guide -n ${NAMESPACE} -o yaml"
    echo -e "   ${BLUE}Helm Chart:${NC} ${SCRIPT_DIR}/../helm/"
    echo
}

# Function to check deployment health
check_deployment_health() {
    print_step "Checking deployment health..."
    
    # Check Portainer pod status
    if kubectl get pods -n ${NAMESPACE} -l app.kubernetes.io/name=portainer | grep -q "Running"; then
        print_status "Portainer pod is running ✓"
    else
        print_warning "Portainer pod may not be running properly"
        kubectl get pods -n ${NAMESPACE} -l app.kubernetes.io/name=portainer
    fi
    
    # Check service endpoints
    if kubectl get endpoints -n ${NAMESPACE} portainer | grep -q ":9000"; then
        print_status "Portainer service endpoints configured ✓"
    else
        print_warning "Portainer service endpoints may not be ready"
        kubectl get endpoints -n ${NAMESPACE} portainer
    fi
    
    # Test Portainer accessibility (if port forwarding is active)
    if pgrep -f "kubectl port-forward.*portainer.*9000:9000" > /dev/null; then
        print_status "Testing Portainer accessibility..."
        if curl -s --max-time 5 http://localhost:9000 > /dev/null; then
            print_status "Portainer is accessible via port forwarding ✓"
        else
            print_warning "Portainer may not be accessible yet. Wait a moment and try again."
        fi
    fi
}

# Function to create troubleshooting guide
create_troubleshooting_guide() {
    print_step "Creating troubleshooting commands..."
    
    cat > "${SCRIPT_DIR}/troubleshoot-portainer.sh" << 'EOF'
#!/bin/bash
# Portainer Troubleshooting Script for Docker Desktop

echo "=== Portainer Troubleshooting ==="

echo "1. Checking Kubernetes context:"
kubectl config current-context

echo -e "\n2. Checking cluster connectivity:"
kubectl cluster-info

echo -e "\n3. Checking Portainer namespace:"
kubectl get namespace portainer

echo -e "\n4. Checking Portainer pods:"
kubectl get pods -n portainer -o wide

echo -e "\n5. Checking Portainer service:"
kubectl get svc -n portainer

echo -e "\n6. Checking Portainer logs (last 20 lines):"
kubectl logs -n portainer deployment/portainer --tail=20

echo -e "\n7. Checking resource usage:"
kubectl top pods -n portainer 2>/dev/null || echo "Metrics server not available"

echo -e "\n8. Checking events:"
kubectl get events -n portainer --sort-by='.lastTimestamp' | tail -10

echo -e "\n9. Port forwarding status:"
pgrep -f "kubectl port-forward.*portainer" || echo "No port forwarding active"

echo -e "\n=== Quick Fixes ==="
echo "Restart port forwarding: kubectl port-forward -n portainer svc/portainer 9000:9000"
echo "Restart Portainer: kubectl rollout restart deployment/portainer -n portainer"
echo "Check logs: kubectl logs -n portainer deployment/portainer -f"
EOF

    chmod +x "${SCRIPT_DIR}/troubleshoot-portainer.sh"
    print_status "Troubleshooting script created: ${SCRIPT_DIR}/troubleshoot-portainer.sh"
}

# Main execution
main() {
    print_status "Starting OpenSim Next Portainer deployment for Docker Desktop..."
    
    check_prerequisites
    install_ingress_controller
    deploy_portainer
    create_opensim_namespace
    setup_helm_repos
    setup_port_forwarding
    check_deployment_health
    create_troubleshooting_guide
    display_access_info
    
    print_status "OpenSim Next Portainer deployment completed successfully! 🎉"
}

# Error handling
trap 'print_error "Script failed at line $LINENO. Exit code: $?"' ERR

# Run main function
main "$@"
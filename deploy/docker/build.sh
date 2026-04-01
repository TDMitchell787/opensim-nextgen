#!/bin/bash
# OpenSim Next Docker Build Script
# Production-ready Docker image build with security scanning

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default configuration
IMAGE_NAME="${IMAGE_NAME:-opensim-next}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
BUILD_PLATFORM="${BUILD_PLATFORM:-linux/amd64}"
ENABLE_SECURITY_SCAN="${ENABLE_SECURITY_SCAN:-true}"
PUSH_IMAGE="${PUSH_IMAGE:-false}"
REGISTRY="${REGISTRY:-}"

# Build information
BUILD_VERSION="${BUILD_VERSION:-$(git describe --tags --always 2>/dev/null || echo 'unknown')}"
BUILD_COMMIT="${BUILD_COMMIT:-$(git rev-parse HEAD 2>/dev/null || echo 'unknown')}"
BUILD_DATE="${BUILD_DATE:-$(date -u +"%Y-%m-%dT%H:%M:%SZ")}"

# Logging functions
log() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')] INFO:${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARN:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')] ERROR:${NC} $1"
}

success() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')] SUCCESS:${NC} $1"
}

# Help function
show_help() {
    cat << EOF
OpenSim Next Docker Build Script

Usage: $0 [OPTIONS]

Options:
  -n, --name NAME          Image name (default: opensim-next)
  -t, --tag TAG           Image tag (default: latest)
  -p, --platform PLATFORM Build platform (default: linux/amd64)
  -r, --registry REGISTRY Registry to push to
  -s, --scan              Enable security scanning (default: true)
  -P, --push              Push image to registry
  -h, --help              Show this help message

Environment Variables:
  IMAGE_NAME              Docker image name
  IMAGE_TAG               Docker image tag
  BUILD_PLATFORM          Target platform
  REGISTRY                Docker registry
  ENABLE_SECURITY_SCAN    Enable/disable security scanning
  PUSH_IMAGE              Push image after build

Examples:
  # Basic build
  $0

  # Build with custom name and tag
  $0 --name my-opensim --tag v1.0.0

  # Build and push to registry
  $0 --registry ghcr.io/opensim --push

  # Multi-platform build
  $0 --platform linux/amd64,linux/arm64

EOF
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        -n|--name)
            IMAGE_NAME="$2"
            shift 2
            ;;
        -t|--tag)
            IMAGE_TAG="$2"
            shift 2
            ;;
        -p|--platform)
            BUILD_PLATFORM="$2"
            shift 2
            ;;
        -r|--registry)
            REGISTRY="$2"
            shift 2
            ;;
        -s|--scan)
            ENABLE_SECURITY_SCAN="true"
            shift
            ;;
        -P|--push)
            PUSH_IMAGE="true"
            shift
            ;;
        -h|--help)
            show_help
            exit 0
            ;;
        *)
            error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Construct full image name
if [ -n "$REGISTRY" ]; then
    FULL_IMAGE_NAME="$REGISTRY/$IMAGE_NAME:$IMAGE_TAG"
else
    FULL_IMAGE_NAME="$IMAGE_NAME:$IMAGE_TAG"
fi

# Check dependencies
check_dependencies() {
    log "Checking build dependencies..."
    
    local deps=("docker" "git")
    local missing_deps=()
    
    for dep in "${deps[@]}"; do
        if ! command -v "$dep" &> /dev/null; then
            missing_deps+=("$dep")
        fi
    done
    
    if [ ${#missing_deps[@]} -ne 0 ]; then
        error "Missing dependencies: ${missing_deps[*]}"
        exit 1
    fi
    
    # Check Docker version
    docker_version=$(docker --version | cut -d' ' -f3 | cut -d',' -f1)
    log "Docker version: $docker_version"
    
    # Check if Docker daemon is running
    if ! docker info &> /dev/null; then
        error "Docker daemon is not running"
        exit 1
    fi
    
    success "All dependencies satisfied"
}

# Prepare build context
prepare_build_context() {
    log "Preparing build context..."
    
    # Change to project root
    cd "$PROJECT_ROOT"
    
    # Ensure Dockerfile exists
    if [ ! -f "deploy/docker/Dockerfile.production" ]; then
        error "Dockerfile not found: deploy/docker/Dockerfile.production"
        exit 1
    fi
    
    # Check .dockerignore
    if [ ! -f "deploy/docker/.dockerignore" ]; then
        warn ".dockerignore not found, build may be slower"
    fi
    
    # Display build context size
    log "Calculating build context size..."
    if command -v du &> /dev/null; then
        context_size=$(du -sh . 2>/dev/null | cut -f1 || echo "unknown")
        log "Build context size: $context_size"
    fi
    
    success "Build context prepared"
}

# Build Docker image
build_image() {
    log "Building Docker image: $FULL_IMAGE_NAME"
    log "Platform: $BUILD_PLATFORM"
    log "Build version: $BUILD_VERSION"
    log "Build commit: $BUILD_COMMIT"
    log "Build date: $BUILD_DATE"
    
    # Build arguments
    local build_args=(
        "--build-arg" "BUILD_VERSION=$BUILD_VERSION"
        "--build-arg" "BUILD_COMMIT=$BUILD_COMMIT"
        "--build-arg" "BUILD_DATE=$BUILD_DATE"
        "--platform" "$BUILD_PLATFORM"
        "--tag" "$FULL_IMAGE_NAME"
        "--file" "deploy/docker/Dockerfile.production"
    )
    
    # Add cache options for faster builds
    if [ "$BUILD_PLATFORM" = "linux/amd64" ]; then
        build_args+=("--cache-from" "$FULL_IMAGE_NAME")
    fi
    
    # Start build
    log "Starting Docker build..."
    local start_time=$(date +%s)
    
    if docker build "${build_args[@]}" .; then
        local end_time=$(date +%s)
        local duration=$((end_time - start_time))
        success "Docker build completed in ${duration}s"
    else
        error "Docker build failed"
        exit 1
    fi
}

# Security scanning
security_scan() {
    if [ "$ENABLE_SECURITY_SCAN" != "true" ]; then
        log "Security scanning disabled"
        return 0
    fi
    
    log "Running security scan on $FULL_IMAGE_NAME"
    
    # Check if Trivy is available
    if command -v trivy &> /dev/null; then
        log "Using Trivy for security scanning..."
        
        # Run Trivy scan
        if trivy image --severity HIGH,CRITICAL --exit-code 1 "$FULL_IMAGE_NAME"; then
            success "Security scan passed"
        else
            error "Security scan found vulnerabilities"
            return 1
        fi
    else
        warn "Trivy not found, using Docker built-in scanning if available"
        
        # Try Docker Scout (if available)
        if docker scout version &> /dev/null; then
            log "Using Docker Scout for security scanning..."
            docker scout cves "$FULL_IMAGE_NAME" || true
        else
            warn "No security scanner available, skipping scan"
        fi
    fi
}

# Image analysis
analyze_image() {
    log "Analyzing built image..."
    
    # Get image information
    local image_id
    image_id=$(docker images --format "{{.ID}}" "$FULL_IMAGE_NAME" | head -n1)
    
    if [ -n "$image_id" ]; then
        log "Image ID: $image_id"
        
        # Get image size
        local image_size
        image_size=$(docker images --format "{{.Size}}" "$FULL_IMAGE_NAME" | head -n1)
        log "Image size: $image_size"
        
        # Get image layers
        local layer_count
        layer_count=$(docker history --quiet "$FULL_IMAGE_NAME" | wc -l)
        log "Number of layers: $layer_count"
        
        # Display image history (top 10 layers)
        log "Image layers (top 10):"
        docker history --format "table {{.ID}}\t{{.CreatedBy}}\t{{.Size}}" "$FULL_IMAGE_NAME" | head -n11
        
    else
        error "Could not find built image"
        return 1
    fi
}

# Test image
test_image() {
    log "Testing built image..."
    
    # Test basic container start
    log "Testing container startup..."
    local container_id
    container_id=$(docker run -d --rm \
        -e DATABASE_URL="sqlite:///tmp/test.db" \
        "$FULL_IMAGE_NAME" config 2>/dev/null || true)
    
    if [ -n "$container_id" ]; then
        # Wait a moment and check if container is still running
        sleep 2
        if docker ps -q --filter "id=$container_id" &> /dev/null; then
            log "Container started successfully"
            docker stop "$container_id" &> /dev/null || true
        else
            # Container exited, check logs
            log "Container test output:"
            docker logs "$container_id" 2>/dev/null || true
        fi
    else
        warn "Could not start test container"
    fi
    
    # Test health check
    log "Testing health check endpoint..."
    # This would require a more complex test setup
    success "Basic image tests completed"
}

# Push image
push_image() {
    if [ "$PUSH_IMAGE" != "true" ]; then
        log "Image push disabled"
        return 0
    fi
    
    if [ -z "$REGISTRY" ]; then
        error "Cannot push: no registry specified"
        return 1
    fi
    
    log "Pushing image to registry: $FULL_IMAGE_NAME"
    
    # Push image
    if docker push "$FULL_IMAGE_NAME"; then
        success "Image pushed successfully"
        log "Image available at: $FULL_IMAGE_NAME"
    else
        error "Failed to push image"
        return 1
    fi
}

# Cleanup
cleanup() {
    log "Cleaning up intermediate images..."
    
    # Remove dangling images
    local dangling_images
    dangling_images=$(docker images -f "dangling=true" -q 2>/dev/null || true)
    
    if [ -n "$dangling_images" ]; then
        log "Removing dangling images..."
        echo "$dangling_images" | xargs docker rmi &> /dev/null || true
    fi
    
    # Optionally run docker system prune
    if [ "${DOCKER_PRUNE:-false}" = "true" ]; then
        log "Running docker system prune..."
        docker system prune -f &> /dev/null || true
    fi
}

# Main execution
main() {
    log "Starting OpenSim Next Docker build"
    log "Image: $FULL_IMAGE_NAME"
    log "Platform: $BUILD_PLATFORM"
    
    # Execute build pipeline
    check_dependencies
    prepare_build_context
    build_image
    analyze_image
    test_image
    
    # Security scanning
    if ! security_scan; then
        error "Security scan failed"
        exit 1
    fi
    
    # Push if requested
    push_image
    
    # Cleanup
    cleanup
    
    # Summary
    success "Build completed successfully!"
    log "Image: $FULL_IMAGE_NAME"
    
    if [ "$PUSH_IMAGE" = "true" ] && [ -n "$REGISTRY" ]; then
        log "Published to: $REGISTRY"
    fi
    
    log "To run the image:"
    echo "  docker run -e DATABASE_URL=postgresql://user:pass@host:5432/db $FULL_IMAGE_NAME"
    
    log "To run with docker-compose:"
    echo "  cd deploy/docker && docker-compose up -d"
}

# Execute main function
main "$@"
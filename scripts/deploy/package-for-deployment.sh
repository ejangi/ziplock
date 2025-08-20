#!/bin/bash
# ZipLock Deployment Package Script
#
# This script creates a comprehensive deployment package for ZipLock
# including the binary, configuration files, logging setup, systemd
# services, and installation scripts.

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PACKAGE_DIR="${PACKAGE_DIR:-$PROJECT_ROOT/target/deployment-package}"
PACKAGE_NAME="${PACKAGE_NAME:-ziplock-deployment}"
VERSION="${VERSION:-$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -n1 | cut -d'"' -f2)}"
BUILD_TARGET="${BUILD_TARGET:-x86_64-unknown-linux-gnu}"
BUILD_MODE="${BUILD_MODE:-release}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# Cleanup function
cleanup() {
    if [[ "${CLEANUP_ON_ERROR:-true}" == "true" && $? -ne 0 ]]; then
        log_warning "Cleaning up due to error..."
        rm -rf "$PACKAGE_DIR" 2>/dev/null || true
    fi
}

trap cleanup EXIT

# Create package directory structure
create_package_structure() {
    log_step "Creating package directory structure..."

    rm -rf "$PACKAGE_DIR"
    mkdir -p "$PACKAGE_DIR"/{bin,config,scripts,systemd,docs,examples}
    mkdir -p "$PACKAGE_DIR"/scripts/{install,uninstall,manage}
    mkdir -p "$PACKAGE_DIR"/config/{logging,application}
    mkdir -p "$PACKAGE_DIR"/docs/{deployment,troubleshooting}

    log_success "Package structure created: $PACKAGE_DIR"
}

# Build the application
build_application() {
    log_step "Building ZipLock application..."

    cd "$PROJECT_ROOT"

    # Clean previous build
    cargo clean

    # Build for the target
    if [[ "$BUILD_MODE" == "release" ]]; then
        log_info "Building optimized release binary..."
        cargo build --release --target "$BUILD_TARGET" --bin ziplock
        BINARY_PATH="target/$BUILD_TARGET/release/ziplock"
    else
        log_info "Building debug binary..."
        cargo build --target "$BUILD_TARGET" --bin ziplock
        BINARY_PATH="target/$BUILD_TARGET/debug/ziplock"
    fi

    if [[ ! -f "$BINARY_PATH" ]]; then
        log_error "Build failed - binary not found at $BINARY_PATH"
        exit 1
    fi

    # Copy binary to package
    cp "$BINARY_PATH" "$PACKAGE_DIR/bin/ziplock"
    chmod +x "$PACKAGE_DIR/bin/ziplock"

    # Get binary info
    BINARY_SIZE=$(du -h "$PACKAGE_DIR/bin/ziplock" | cut -f1)
    log_success "Binary built and packaged (size: $BINARY_SIZE)"
}

# Package configuration files
package_configurations() {
    log_step "Packaging configuration files..."

    # Main logging configuration
    if [[ -f "$PROJECT_ROOT/config/logging.yaml" ]]; then
        cp "$PROJECT_ROOT/config/logging.yaml" "$PACKAGE_DIR/config/logging/"
        log_info "Copied logging configuration"
    else
        log_warning "Logging configuration not found, creating default..."
        create_default_logging_config
    fi

    # Application configuration template
    create_application_config_template

    # Environment-specific configs
    create_environment_configs

    log_success "Configuration files packaged"
}

# Create default logging configuration if missing
create_default_logging_config() {
    cat > "$PACKAGE_DIR/config/logging/logging.yaml" << 'EOF'
# ZipLock Logging Configuration

default:
  console:
    enabled: true
    level: "INFO"
    timestamps: true
    colors: auto
    format: "compact"
  file:
    enabled: true
    level: "DEBUG"
    directory: "/var/log/ziplock"
    filename: "ziplock.log"
    timestamps: true
    format: "detailed"
  rotation:
    enabled: true
    max_file_size: "10MB"
    max_files: 5
    compress: true
  features:
    thread_ids: false
    source_location: false
    performance_tracking: false

production:
  console:
    enabled: false
    level: "WARN"
    timestamps: false
    colors: never
    format: "json"
  file:
    enabled: true
    level: "INFO"
    directory: "/var/log/ziplock"
    filename: "ziplock.log"
    timestamps: true
    format: "json"
  rotation:
    enabled: true
    max_file_size: "50MB"
    max_files: 10
    compress: true
  features:
    thread_ids: false
    source_location: false
    performance_tracking: false

systemd:
  console:
    enabled: false
    level: "OFF"
    timestamps: false
    colors: never
    format: "json"
  file:
    enabled: true
    level: "INFO"
    directory: "/var/log/ziplock"
    filename: "ziplock.log"
    timestamps: true
    format: "json"
  rotation:
    enabled: true
    max_file_size: "100MB"
    max_files: 20
    compress: true
  features:
    thread_ids: false
    source_location: false
    performance_tracking: false
EOF
}

# Create application configuration template
create_application_config_template() {
    cat > "$PACKAGE_DIR/config/application/ziplock.yaml" << 'EOF'
# ZipLock Application Configuration Template
# Copy this file to /etc/ziplock/config.yaml and customize as needed

# Application settings
application:
  name: "ZipLock Password Manager"
  version: "0.3.0"

  # Environment (development, production, testing)
  environment: "production"

  # Data directories
  data_dir: "/var/lib/ziplock"
  cache_dir: "/var/cache/ziplock"
  config_dir: "/etc/ziplock"

# Security settings
security:
  # Session timeout in seconds (0 = no timeout)
  session_timeout: 900

  # Auto-lock timeout in seconds (0 = disabled)
  auto_lock_timeout: 300

  # Maximum failed attempts before lockout
  max_failed_attempts: 5

  # Lockout duration in seconds
  lockout_duration: 300

# Performance settings
performance:
  # Worker thread pool size (0 = auto-detect)
  worker_threads: 0

  # Memory pool size for crypto operations
  crypto_memory_pool: "64MB"

  # Archive cache size
  archive_cache_size: "32MB"

# Logging (can override logging.yaml)
logging:
  # Override log level if needed
  # level: "INFO"

  # Override log directory if needed
  # directory: "/var/log/ziplock"

# Feature flags
features:
  # Enable experimental features
  experimental: false

  # Enable performance monitoring
  performance_monitoring: false

  # Enable audit logging
  audit_logging: true
EOF
}

# Create environment-specific configurations
create_environment_configs() {
    # Development environment
    cat > "$PACKAGE_DIR/config/application/development.yaml" << 'EOF'
# Development Environment Configuration
application:
  environment: "development"
  data_dir: "./data"
  cache_dir: "./cache"
  config_dir: "./config"

security:
  session_timeout: 0
  auto_lock_timeout: 0

logging:
  level: "DEBUG"
  directory: "./logs"

features:
  experimental: true
  performance_monitoring: true
  audit_logging: true
EOF

    # Production environment
    cat > "$PACKAGE_DIR/config/application/production.yaml" << 'EOF'
# Production Environment Configuration
application:
  environment: "production"
  data_dir: "/var/lib/ziplock"
  cache_dir: "/var/cache/ziplock"
  config_dir: "/etc/ziplock"

security:
  session_timeout: 900
  auto_lock_timeout: 300
  max_failed_attempts: 3
  lockout_duration: 600

logging:
  level: "INFO"
  directory: "/var/log/ziplock"

features:
  experimental: false
  performance_monitoring: false
  audit_logging: true
EOF
}

# Package scripts
package_scripts() {
    log_step "Packaging installation and management scripts..."

    # Main installation script
    if [[ -f "$PROJECT_ROOT/scripts/deploy/setup-logging.sh" ]]; then
        cp "$PROJECT_ROOT/scripts/deploy/setup-logging.sh" "$PACKAGE_DIR/scripts/install/"
        chmod +x "$PACKAGE_DIR/scripts/install/setup-logging.sh"
    fi

    if [[ -f "$PROJECT_ROOT/scripts/deploy/manage-service.sh" ]]; then
        cp "$PROJECT_ROOT/scripts/deploy/manage-service.sh" "$PACKAGE_DIR/scripts/manage/"
        chmod +x "$PACKAGE_DIR/scripts/manage/manage-service.sh"
    fi

    # Create main installer script
    create_main_installer

    # Create uninstaller script
    create_uninstaller

    # Create post-install configuration script
    create_post_install_config

    log_success "Scripts packaged"
}

# Create main installer script
create_main_installer() {
    cat > "$PACKAGE_DIR/scripts/install/install.sh" << 'EOF'
#!/bin/bash
# ZipLock Main Installation Script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PACKAGE_DIR="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Default installation paths
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr}"
CONFIG_DIR="${CONFIG_DIR:-/etc/ziplock}"
DATA_DIR="${DATA_DIR:-/var/lib/ziplock}"
LOG_DIR="${LOG_DIR:-/var/log/ziplock}"
CACHE_DIR="${CACHE_DIR:-/var/cache/ziplock}"

# User and group
ZIPLOCK_USER="${ZIPLOCK_USER:-ziplock}"
ZIPLOCK_GROUP="${ZIPLOCK_GROUP:-ziplock}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check root privileges
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root (use sudo)"
    exit 1
fi

log_info "Installing ZipLock Password Manager..."

# Install binary
log_info "Installing binary to $INSTALL_PREFIX/bin/"
cp "$PACKAGE_DIR/bin/ziplock" "$INSTALL_PREFIX/bin/"
chmod +x "$INSTALL_PREFIX/bin/ziplock"

# Create directories
log_info "Creating directories..."
mkdir -p "$CONFIG_DIR" "$DATA_DIR" "$LOG_DIR" "$CACHE_DIR"

# Install configurations
log_info "Installing configuration files..."
cp -r "$PACKAGE_DIR/config/"* "$CONFIG_DIR/"

# Create user and group
if ! getent group "$ZIPLOCK_GROUP" >/dev/null 2>&1; then
    groupadd --system "$ZIPLOCK_GROUP"
    log_success "Created group: $ZIPLOCK_GROUP"
fi

if ! getent passwd "$ZIPLOCK_USER" >/dev/null 2>&1; then
    useradd --system --gid "$ZIPLOCK_GROUP" \
            --home-dir "$DATA_DIR" \
            --shell /bin/false \
            --comment "ZipLock Password Manager" \
            "$ZIPLOCK_USER"
    log_success "Created user: $ZIPLOCK_USER"
fi

# Set permissions
chown -R "$ZIPLOCK_USER:$ZIPLOCK_GROUP" "$DATA_DIR" "$LOG_DIR" "$CACHE_DIR"
chmod 750 "$DATA_DIR" "$LOG_DIR" "$CACHE_DIR"
chmod 755 "$CONFIG_DIR"

# Run logging setup
if [[ -f "$PACKAGE_DIR/scripts/install/setup-logging.sh" ]]; then
    log_info "Setting up logging and systemd service..."
    bash "$PACKAGE_DIR/scripts/install/setup-logging.sh"
fi

log_success "ZipLock installation completed!"
log_info "Configuration files: $CONFIG_DIR"
log_info "Data directory: $DATA_DIR"
log_info "Log directory: $LOG_DIR"
log_info ""
log_info "To start ZipLock service:"
log_info "  systemctl start ziplock"
log_info ""
log_info "To enable automatic startup:"
log_info "  systemctl enable ziplock"
log_info ""
log_info "To check service status:"
log_info "  systemctl status ziplock"
EOF

    chmod +x "$PACKAGE_DIR/scripts/install/install.sh"
}

# Create uninstaller script
create_uninstaller() {
    cat > "$PACKAGE_DIR/scripts/uninstall/uninstall.sh" << 'EOF'
#!/bin/bash
# ZipLock Uninstaller Script

set -euo pipefail

# Installation paths
INSTALL_PREFIX="${INSTALL_PREFIX:-/usr}"
CONFIG_DIR="${CONFIG_DIR:-/etc/ziplock}"
DATA_DIR="${DATA_DIR:-/var/lib/ziplock}"
LOG_DIR="${LOG_DIR:-/var/log/ziplock}"
CACHE_DIR="${CACHE_DIR:-/var/cache/ziplock}"

# User and group
ZIPLOCK_USER="${ZIPLOCK_USER:-ziplock}"
ZIPLOCK_GROUP="${ZIPLOCK_GROUP:-ziplock}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Check root privileges
if [[ $EUID -ne 0 ]]; then
    log_error "This script must be run as root (use sudo)"
    exit 1
fi

log_warning "This will completely remove ZipLock and all its data!"
read -p "Are you sure you want to continue? (y/N): " -n 1 -r
echo

if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    log_info "Uninstallation cancelled"
    exit 0
fi

log_info "Uninstalling ZipLock Password Manager..."

# Stop and disable service
if systemctl list-unit-files ziplock.service --no-legend --no-pager | grep -q ziplock.service; then
    log_info "Stopping and disabling service..."
    systemctl stop ziplock.service || true
    systemctl disable ziplock.service || true
    rm -f /etc/systemd/system/ziplock.service
    systemctl daemon-reload
fi

# Remove binary
if [[ -f "$INSTALL_PREFIX/bin/ziplock" ]]; then
    rm -f "$INSTALL_PREFIX/bin/ziplock"
    log_success "Binary removed"
fi

# Remove configuration files
if [[ -d "$CONFIG_DIR" ]]; then
    rm -rf "$CONFIG_DIR"
    log_success "Configuration removed"
fi

# Ask about data removal
read -p "Remove all data and logs? This cannot be undone! (y/N): " -n 1 -r
echo

if [[ $REPLY =~ ^[Yy]$ ]]; then
    rm -rf "$DATA_DIR" "$LOG_DIR" "$CACHE_DIR"
    log_success "Data and logs removed"

    # Remove user and group
    if getent passwd "$ZIPLOCK_USER" >/dev/null 2>&1; then
        userdel "$ZIPLOCK_USER" || true
        log_success "User removed"
    fi

    if getent group "$ZIPLOCK_GROUP" >/dev/null 2>&1; then
        groupdel "$ZIPLOCK_GROUP" || true
        log_success "Group removed"
    fi
fi

# Remove logrotate config
if [[ -f "/etc/logrotate.d/ziplock" ]]; then
    rm -f /etc/logrotate.d/ziplock
    log_success "Logrotate configuration removed"
fi

log_success "ZipLock uninstallation completed!"
EOF

    chmod +x "$PACKAGE_DIR/scripts/uninstall/uninstall.sh"
}

# Create post-install configuration script
create_post_install_config() {
    cat > "$PACKAGE_DIR/scripts/install/post-install-config.sh" << 'EOF'
#!/bin/bash
# ZipLock Post-Installation Configuration Script

set -euo pipefail

CONFIG_DIR="${CONFIG_DIR:-/etc/ziplock}"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[WARNING]${NC} $1"; }

log_info "ZipLock Post-Installation Configuration"
echo

# Check if running as root
if [[ $EUID -eq 0 ]]; then
    log_warning "Running as root. Configuration will be system-wide."
else
    log_info "Running as user. Configuration will be user-specific."
fi

# Environment selection
echo "Select deployment environment:"
echo "1) Development"
echo "2) Production (default)"
echo "3) Testing"
read -p "Choice [2]: " ENV_CHOICE

case ${ENV_CHOICE:-2} in
    1)
        ENVIRONMENT="development"
        ;;
    3)
        ENVIRONMENT="testing"
        ;;
    *)
        ENVIRONMENT="production"
        ;;
esac

log_info "Selected environment: $ENVIRONMENT"

# Copy environment-specific configuration
if [[ -f "$CONFIG_DIR/application/$ENVIRONMENT.yaml" ]]; then
    cp "$CONFIG_DIR/application/$ENVIRONMENT.yaml" "$CONFIG_DIR/config.yaml"
    log_success "Configuration applied for $ENVIRONMENT environment"
fi

# Logging configuration
echo
log_info "Configuring logging..."

if [[ -f "$CONFIG_DIR/logging/logging.yaml" ]]; then
    echo "Current logging environments available:"
    grep '^[a-zA-Z]' "$CONFIG_DIR/logging/logging.yaml" | grep ':$' | sed 's/:$//' | sed 's/^/  - /'

    read -p "Use '$ENVIRONMENT' logging profile? [Y/n]: " USE_ENV_LOGGING

    if [[ ${USE_ENV_LOGGING:-Y} =~ ^[Yy]$ ]]; then
        export ZIPLOCK_LOG_ENV="$ENVIRONMENT"
        log_success "Logging configured for $ENVIRONMENT environment"
    fi
fi

# Service configuration
echo
log_info "Service configuration..."

if systemctl list-unit-files ziplock.service --no-legend --no-pager | grep -q ziplock.service; then
    read -p "Enable ZipLock service for automatic startup? [Y/n]: " ENABLE_SERVICE

    if [[ ${ENABLE_SERVICE:-Y} =~ ^[Yy]$ ]]; then
        systemctl enable ziplock.service
        log_success "Service enabled for automatic startup"

        read -p "Start ZipLock service now? [Y/n]: " START_SERVICE

        if [[ ${START_SERVICE:-Y} =~ ^[Yy]$ ]]; then
            systemctl start ziplock.service
            log_success "Service started"

            echo
            log_info "Service status:"
            systemctl status ziplock.service --no-pager -l || true
        fi
    fi
fi

echo
log_success "Post-installation configuration completed!"
log_info "Configuration file: $CONFIG_DIR/config.yaml"
log_info "Logging configuration: $CONFIG_DIR/logging/logging.yaml"
echo
log_info "Useful commands:"
echo "  systemctl status ziplock    # Check service status"
echo "  systemctl logs ziplock      # View service logs"
echo "  tail -f /var/log/ziplock/ziplock.log  # Follow application logs"
EOF

    chmod +x "$PACKAGE_DIR/scripts/install/post-install-config.sh"
}

# Package systemd files
package_systemd_files() {
    log_step "Packaging systemd service files..."

    # Main service file (generated by setup-logging.sh, but we'll create a template)
    cat > "$PACKAGE_DIR/systemd/ziplock.service" << 'EOF'
[Unit]
Description=ZipLock Password Manager
Documentation=https://github.com/ejangi/ziplock
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=/usr/bin/ziplock
Restart=always
RestartSec=10
RestartPreventExitStatus=1

# User and group
User=ziplock
Group=ziplock

# Working directory
WorkingDirectory=/var/lib/ziplock

# Environment
Environment=ZIPLOCK_ENV=production
Environment=RUST_LOG=info
Environment=ZIPLOCK_LOG_DIR=/var/log/ziplock

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=ziplock

# Security settings
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/log/ziplock
ReadWritePaths=/var/lib/ziplock
ReadWritePaths=/var/cache/ziplock

[Install]
WantedBy=multi-user.target
EOF

    log_success "Systemd files packaged"
}

# Package documentation
package_documentation() {
    log_step "Packaging documentation..."

    # Main README
    cat > "$PACKAGE_DIR/README.md" << EOF
# ZipLock Deployment Package

Version: $VERSION
Build Target: $BUILD_TARGET
Build Mode: $BUILD_MODE
Package Date: $(date -u +"%Y-%m-%d %H:%M:%S UTC")

## Contents

- \`bin/ziplock\` - Main application binary
- \`config/\` - Configuration files and templates
- \`scripts/install/\` - Installation scripts
- \`scripts/uninstall/\` - Uninstallation scripts
- \`scripts/manage/\` - Service management scripts
- \`systemd/\` - Systemd service files
- \`docs/\` - Documentation

## Quick Installation

1. Extract this package to a temporary directory
2. Run as root: \`./scripts/install/install.sh\`
3. Configure: \`./scripts/install/post-install-config.sh\`
4. Start service: \`systemctl start ziplock\`

## Manual Installation

See \`docs/deployment/INSTALL.md\` for detailed instructions.

## Service Management

Use the included service management script:
\`./scripts/manage/manage-service.sh --help\`

## Configuration

- Main config: \`/etc/ziplock/config.yaml\`
- Logging config: \`/etc/ziplock/logging/logging.yaml\`
- Service logs: \`/var/log/ziplock/ziplock.log\`
- Systemd logs: \`journalctl -u ziplock\`

## Support

For issues and documentation, visit:
https://github.com/ejangi/ziplock
EOF

    # Installation guide
    create_install_guide

    # Troubleshooting guide
    create_troubleshooting_guide

    log_success "Documentation packaged"
}

# Create installation guide
create_install_guide() {
    cat > "$PACKAGE_DIR/docs/deployment/INSTALL.md" << 'EOF'
# ZipLock Installation Guide

## Prerequisites

- Linux system with systemd
- Root access (sudo)
- Minimum 100MB disk space
- glibc 2.27 or newer

## Automated Installation

### Quick Install
```bash
sudo ./scripts/install/install.sh
./scripts/install/post-install-config.sh
```

### Custom Installation
```bash
# Set custom paths
export INSTALL_PREFIX=/opt/ziplock
export CONFIG_DIR=/opt/ziplock/etc
export DATA_DIR=/opt/ziplock/var/lib
export LOG_DIR=/opt/ziplock/var/log

sudo ./scripts/install/install.sh
```

## Manual Installation

### 1. Install Binary
```bash
sudo cp bin/ziplock /usr/bin/
sudo chmod +x /usr/bin/ziplock
```

### 2. Create User and Directories
```bash
sudo groupadd --system ziplock
sudo useradd --system --gid ziplock --home-dir /var/lib/ziplock \
    --shell /bin/false --comment "ZipLock Password Manager" ziplock

sudo mkdir -p /etc/ziplock /var/lib/ziplock /var/log/ziplock /var/cache/ziplock
sudo chown ziplock:ziplock /var/lib/ziplock /var/log/ziplock /var/cache/ziplock
sudo chmod 750 /var/lib/ziplock /var/log/ziplock /var/cache/ziplock
```

### 3. Install Configuration
```bash
sudo cp -r config/* /etc/ziplock/
sudo chmod 644 /etc/ziplock/config.yaml
```

### 4. Install Service
```bash
sudo cp systemd/ziplock.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable ziplock
```

### 5. Configure Logging
```bash
sudo cp config/logging/logging.yaml /etc/ziplock/logging/
sudo bash scripts/install/setup-logging.sh
```

## Post-Installation

### Start Service
```bash
sudo systemctl start ziplock
sudo systemctl status ziplock
```

### Check Logs
```bash
# Systemd logs
sudo journalctl -u ziplock -f

# Application logs
sudo tail -f /var/log/ziplock/ziplock.log
```

### Configuration
Edit `/etc/ziplock/config.yaml` to customize settings.

## Verification

### Test Installation
```bash
# Check binary
/usr/bin/ziplock --version

# Check service
systemctl is-active ziplock
systemctl is-enabled ziplock

# Check logs
ls -la /var/log/ziplock/
```

### Health Check
```bash
sudo ./scripts/manage/manage-service.sh test
```

## Troubleshooting

See `docs/troubleshooting/TROUBLESHOOTING.md` for common issues and solutions.
EOF
}

# Create troubleshooting guide
create_troubleshooting_guide() {
    cat > "$PACKAGE_DIR/docs/troubleshooting/TROUBLESHOOTING.md" << 'EOF'
# ZipLock Troubleshooting Guide

## Service Issues

### Service Won't Start
```bash
# Check service status
sudo systemctl status ziplock

# Check logs
sudo journalctl -u ziplock -n 50

# Test binary directly
sudo -u ziplock /usr/bin/ziplock --version
```

**Common causes:**
- Missing dependencies
- Permission issues
- Configuration errors
- Port conflicts

### Service Crashes
```bash
# Check crash logs
sudo journalctl -u ziplock --since "1 hour ago"

# Check application logs
sudo tail -100 /var/log/ziplock/ziplock.log

# Run in debug mode
sudo -u ziplock RUST_LOG=debug /usr/bin/ziplock
```

## Logging Issues

### No Log Files
```bash
# Check directory permissions
ls -la /var/log/ziplock/

# Check logging configuration
cat /etc/ziplock/logging/logging.yaml

# Test logging setup
sudo ./scripts/dev/test-logging.sh --local-only
```

### Log Rotation Problems
```bash
# Test logrotate configuration
sudo logrotate -d /etc/logrotate.d/ziplock

# Force rotation
sudo logrotate -f /etc/logrotate.d/ziplock

# Check logrotate logs
sudo journalctl -u logrotate
```

## Permission Issues

### Access Denied Errors
```bash
# Fix directory permissions
sudo chown -R ziplock:ziplock /var/lib/ziplock /var/log/ziplock
sudo chmod 750 /var/lib/ziplock /var/log/ziplock

# Check service file permissions
ls -la /etc/systemd/system/ziplock.service
```

### Configuration Access Issues
```bash
# Fix config permissions
sudo chown root:ziplock /etc/ziplock/config.yaml
sudo chmod 640 /etc/ziplock/config.yaml
```

## Configuration Issues

### Invalid Configuration
```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('/etc/ziplock/config.yaml'))"

# Reset to defaults
sudo cp config/application/production.yaml /etc/ziplock/config.yaml
```

### Environment Detection Issues
```bash
# Check environment variables
sudo systemctl show ziplock | grep Environment

# Override environment
sudo systemctl edit ziplock
# Add: Environment=ZIPLOCK_ENV=production
```

## Performance Issues

### High Memory Usage
```bash
# Check process memory
ps aux | grep ziplock

# Monitor resources
sudo ./scripts/manage/manage-service.sh monitor
```

### Slow Startup
```bash
# Check startup time
systemd-analyze blame | grep ziplock

# Enable debug logging
sudo systemctl edit ziplock
# Add: Environment=RUST_LOG=debug
```

## Network Issues

### Port Conflicts
```bash
# Check listening ports
sudo netstat -tlnp | grep ziplock

# Change port in configuration
sudo nano /etc/ziplock/config.yaml
```

## Recovery Procedures

### Complete Reset
```bash
# Stop service
sudo systemctl stop ziplock

# Backup data
sudo cp -r /var/lib/ziplock /var/lib/ziplock.backup

# Reset configuration
sudo cp config/application/production.yaml /etc/ziplock/config.yaml

# Restart service
sudo systemctl start ziplock
```

### Reinstall Service
```bash
# Remove service
sudo systemctl stop ziplock
sudo systemctl disable ziplock
sudo rm /etc/systemd/system/ziplock.service

# Reinstall
sudo bash scripts/install/setup-logging.sh
sudo systemctl enable ziplock
sudo systemctl start ziplock
```

## Getting Help

### Collect Debug Information
```bash
# Create debug package
DEBUG_DIR="/tmp/ziplock-debug-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$DEBUG_DIR"

# Copy relevant files
sudo cp /etc/systemd/system/ziplock.service "$DEBUG_DIR/"
sudo cp -r /etc/ziplock "$DEBUG_DIR/"
sudo journalctl -u ziplock > "$DEBUG_DIR/systemd.log"
sudo cp -r /var/log/ziplock "$DEBUG_DIR/" 2>/dev/null || true

# System information
uname -a > "$DEBUG_DIR/system-info.txt"
systemctl --version >> "$DEBUG_DIR/system-info.txt"
df -h >> "$DEBUG_DIR/system-info.txt"

echo "Debug information collected in: $DEBUG_DIR"
```

### Report Issues

1. Visit: https://github.com/ejangi/ziplock/issues
2. Include system information and debug logs
3. Describe expected vs actual behavior
4. Provide reproduction steps
EOF
}

# Create examples and additional documentation
+package_examples() {
+    log_step "Packaging examples and additional documentation..."
+
+    # Create systemd override example
+    mkdir -p "$PACKAGE_DIR/examples/systemd"
+    cat > "$PACKAGE_DIR/examples/systemd/override-example.conf" << 'EOF'
+# Example systemd service override
+# Place in: /etc/systemd/system/ziplock.service.d/override.conf
+
+[Service]
+# Override environment variables
+Environment=ZIPLOCK_ENV=production
+Environment=RUST_LOG=info
+Environment=ZIPLOCK_LOG_DIR=/var/log/ziplock
+
+# Additional security settings
+PrivateNetwork=yes
+ProtectClock=yes
+ProtectHostname=yes
+
+# Resource limits
+MemoryMax=256M
+TasksMax=100
+EOF
+
+    # Create logging examples
+    mkdir -p "$PACKAGE_DIR/examples/logging"
+    cat > "$PACKAGE_DIR/examples/logging/custom-logging.yaml" << 'EOF'
+# Example custom logging configuration
+custom:
+  console:
+    enabled: true
+    level: "INFO"
+    timestamps: true
+    colors: auto
+    format: "pretty"
+  file:
+    enabled: true
+    level: "DEBUG"
+    directory: "/var/log/ziplock"
+    filename: "ziplock-custom.log"
+    timestamps: true
+    format: "json"
+  rotation:
+    enabled: true
+    max_file_size: "25MB"
+    max_files: 7
+    compress: true
+  features:
+    thread_ids: false
+    source_location: false
+    performance_tracking: true
+EOF
+
+    log_success "Examples packaged"
+}
+
+# Validate the package
+validate_package() {
+    log_step "Validating deployment package..."
+
+    local errors=0
+
+    # Check required files
+    local required_files=(
+        "bin/ziplock"
+        "config/logging/logging.yaml"
+        "config/application/ziplock.yaml"
+        "scripts/install/install.sh"
+        "scripts/uninstall/uninstall.sh"
+        "systemd/ziplock.service"
+        "README.md"
+    )
+
+    for file in "${required_files[@]}"; do
+        if [[ ! -f "$PACKAGE_DIR/$file" ]]; then
+            log_error "Missing required file: $file"
+            ((errors++))
+        fi
+    done
+
+    # Check executable permissions
+    local executable_files=(
+        "bin/ziplock"
+        "scripts/install/install.sh"
+        "scripts/uninstall/uninstall.sh"
+        "scripts/install/setup-logging.sh"
+        "scripts/manage/manage-service.sh"
+        "scripts/install/post-install-config.sh"
+    )
+
+    for file in "${executable_files[@]}"; do
+        if [[ -f "$PACKAGE_DIR/$file" && ! -x "$PACKAGE_DIR/$file" ]]; then
+            log_error "File not executable: $file"
+            ((errors++))
+        fi
+    done
+
+    # Check binary
+    if [[ -f "$PACKAGE_DIR/bin/ziplock" ]]; then
+        if ! ldd "$PACKAGE_DIR/bin/ziplock" >/dev/null 2>&1; then
+            log_warning "Binary may have missing dependencies"
+        fi
+
+        local binary_arch=$(file "$PACKAGE_DIR/bin/ziplock" | grep -o 'x86-64\|i386\|ARM\|aarch64')
+        log_info "Binary architecture: ${binary_arch:-unknown}"
+    fi
+
+    # Validate YAML files
+    if command -v python3 >/dev/null 2>&1; then
+        for yaml_file in $(find "$PACKAGE_DIR" -name "*.yaml" -type f); do
+            if ! python3 -c "import yaml; yaml.safe_load(open('$yaml_file'))" >/dev/null 2>&1; then
+                log_error "Invalid YAML syntax: $yaml_file"
+                ((errors++))
+            fi
+        done
+    else
+        log_warning "Python3 not available - skipping YAML validation"
+    fi
+
+    if [[ $errors -eq 0 ]]; then
+        log_success "Package validation passed"
+        return 0
+    else
+        log_error "Package validation failed with $errors errors"
+        return 1
+    fi
+}
+
+# Create package archive
+create_package_archive() {
+    log_step "Creating deployment archive..."
+
+    cd "$(dirname "$PACKAGE_DIR")"
+
+    local archive_name="${PACKAGE_NAME}-${VERSION}-${BUILD_TARGET}.tar.gz"
+    local archive_path="$(dirname "$PACKAGE_DIR")/$archive_name"
+
+    # Create tar.gz archive
+    tar -czf "$archive_path" -C "$(dirname "$PACKAGE_DIR")" "$(basename "$PACKAGE_DIR")"
+
+    # Create checksums
+    cd "$(dirname "$archive_path")"
+    sha256sum "$archive_name" > "${archive_name}.sha256"
+    md5sum "$archive_name" > "${archive_name}.md5"
+
+    local archive_size=$(du -h "$archive_path" | cut -f1)
+    log_success "Package archive created: $archive_path ($archive_size)"
+
+    # Display package information
+    echo
+    log_info "Package Information:"
+    echo "  Name: $PACKAGE_NAME"
+    echo "  Version: $VERSION"
+    echo "  Target: $BUILD_TARGET"
+    echo "  Build Mode: $BUILD_MODE"
+    echo "  Archive: $archive_path"
+    echo "  Size: $archive_size"
+    echo "  SHA256: $(cat "${archive_path}.sha256" | cut -d' ' -f1)"
+    echo
+}
+
+# Display usage information
+usage() {
+    echo "Usage: $0 [OPTIONS]"
+    echo
+    echo "Create a comprehensive deployment package for ZipLock"
+    echo
+    echo "Options:"
+    echo "  --package-dir DIR     Output directory (default: target/deployment-package)"
+    echo "  --package-name NAME   Package name (default: ziplock-deployment)"
+    echo "  --version VERSION     Package version (default: from Cargo.toml)"
+    echo "  --target TARGET       Build target (default: x86_64-unknown-linux-gnu)"
+    echo "  --build-mode MODE     Build mode: release|debug (default: release)"
+    echo "  --skip-build          Skip building the application"
+    echo "  --skip-validation     Skip package validation"
+    echo "  --skip-archive        Skip creating archive"
+    echo "  --cleanup-on-error    Clean up package dir on error (default: true)"
+    echo "  --help                Show this help message"
+    echo
+    echo "Environment Variables:"
+    echo "  PACKAGE_DIR           Output directory"
+    echo "  PACKAGE_NAME          Package name"
+    echo "  VERSION               Package version"
+    echo "  BUILD_TARGET          Build target"
+    echo "  BUILD_MODE            Build mode"
+    echo "  CLEANUP_ON_ERROR      Clean up on error (true/false)"
+}
+
+# Parse command line arguments
+SKIP_BUILD=false
+SKIP_VALIDATION=false
+SKIP_ARCHIVE=false
+
+while [[ $# -gt 0 ]]; do
+    case $1 in
+        --package-dir)
+            PACKAGE_DIR="$2"
+            shift 2
+            ;;
+        --package-name)
+            PACKAGE_NAME="$2"
+            shift 2
+            ;;
+        --version)
+            VERSION="$2"
+            shift 2
+            ;;
+        --target)
+            BUILD_TARGET="$2"
+            shift 2
+            ;;
+        --build-mode)
+            BUILD_MODE="$2"
+            shift 2
+            ;;
+        --skip-build)
+            SKIP_BUILD=true
+            shift
+            ;;
+        --skip-validation)
+            SKIP_VALIDATION=true
+            shift
+            ;;
+        --skip-archive)
+            SKIP_ARCHIVE=true
+            shift
+            ;;
+        --cleanup-on-error)
+            export CLEANUP_ON_ERROR=true
+            shift
+            ;;
+        --help)
+            usage
+            exit 0
+            ;;
+        *)
+            log_error "Unknown option: $1"
+            usage
+            exit 1
+            ;;
+    esac
+done
+
+# Main execution
+main() {
+    log_info "Creating ZipLock deployment package..."
+    log_info "Package: $PACKAGE_NAME v$VERSION"
+    log_info "Target: $BUILD_TARGET ($BUILD_MODE)"
+    log_info "Output: $PACKAGE_DIR"
+
+    create_package_structure
+
+    if [[ "$SKIP_BUILD" != "true" ]]; then
+        build_application
+    else
+        log_info "Skipping build - using existing binary"
+        if [[ -f "target/$BUILD_TARGET/$BUILD_MODE/ziplock" ]]; then
+            cp "target/$BUILD_TARGET/$BUILD_MODE/ziplock" "$PACKAGE_DIR/bin/ziplock"
+            chmod +x "$PACKAGE_DIR/bin/ziplock"
+        else
+            log_error "No existing binary found at target/$BUILD_TARGET/$BUILD_MODE/ziplock"
+            exit 1
+        fi
+    fi
+
+    package_configurations
+    package_scripts
+    package_systemd_files
+    package_documentation
+    package_examples
+
+    if [[ "$SKIP_VALIDATION" != "true" ]]; then
+        if ! validate_package; then
+            log_error "Package validation failed"
+            exit 1
+        fi
+    else
+        log_info "Skipping package validation"
+    fi
+
+    if [[ "$SKIP_ARCHIVE" != "true" ]]; then
+        create_package_archive
+    else
+        log_info "Skipping archive creation"
+        log_success "Package ready at: $PACKAGE_DIR"
+    fi
+
+    log_success "Deployment package creation completed!"
+}
+
+# Run main function
+main "$@"

#!/bin/bash
# ZipLock Logging Setup Script
#
# This script sets up logging and log rotation for ZipLock when deployed
# as a system service or standalone application on Linux systems.

set -euo pipefail

# Configuration
ZIPLOCK_USER="${ZIPLOCK_USER:-ziplock}"
ZIPLOCK_GROUP="${ZIPLOCK_GROUP:-ziplock}"
LOG_DIR="${ZIPLOCK_LOG_DIR:-/var/log/ziplock}"
SYSTEMD_SERVICE_DIR="/etc/systemd/system"
LOGROTATE_CONFIG_DIR="/etc/logrotate.d"
ZIPLOCK_BIN="${ZIPLOCK_BIN:-/usr/bin/ziplock}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
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

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Create system user and group for ZipLock
create_user_group() {
    log_info "Creating system user and group for ZipLock..."

    # Create group if it doesn't exist
    if ! getent group "$ZIPLOCK_GROUP" >/dev/null 2>&1; then
        groupadd --system "$ZIPLOCK_GROUP"
        log_success "Created group: $ZIPLOCK_GROUP"
    else
        log_info "Group $ZIPLOCK_GROUP already exists"
    fi

    # Create user if it doesn't exist
    if ! getent passwd "$ZIPLOCK_USER" >/dev/null 2>&1; then
        useradd --system --gid "$ZIPLOCK_GROUP" \
                --home-dir /var/lib/ziplock \
                --shell /bin/false \
                --comment "ZipLock Password Manager" \
                "$ZIPLOCK_USER"
        log_success "Created user: $ZIPLOCK_USER"
    else
        log_info "User $ZIPLOCK_USER already exists"
    fi
}

# Setup log directory with proper permissions
setup_log_directory() {
    log_info "Setting up log directory: $LOG_DIR"

    # Create log directory
    mkdir -p "$LOG_DIR"

    # Set ownership and permissions
    chown "$ZIPLOCK_USER:$ZIPLOCK_GROUP" "$LOG_DIR"
    chmod 750 "$LOG_DIR"

    log_success "Log directory created with proper permissions"
}

# Generate systemd service file
generate_systemd_service() {
    log_info "Creating systemd service file..."

    cat > "$SYSTEMD_SERVICE_DIR/ziplock.service" << EOF
[Unit]
Description=ZipLock Password Manager
Documentation=https://github.com/ejangi/ziplock
After=network.target
Wants=network.target

[Service]
Type=simple
ExecStart=$ZIPLOCK_BIN
Restart=always
RestartSec=10
RestartPreventExitStatus=1

# User and group
User=$ZIPLOCK_USER
Group=$ZIPLOCK_GROUP

# Working directory
WorkingDirectory=/var/lib/ziplock

# Environment
Environment=ZIPLOCK_ENV=production
Environment=RUST_LOG=info
Environment=ZIPLOCK_LOG_DIR=$LOG_DIR

# Logging
StandardOutput=journal
StandardError=journal
SyslogIdentifier=ziplock

# Log rate limiting (disable to capture all logs)
LogRateLimitIntervalSec=0
LogRateLimitBurst=0

# Security settings
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ProtectKernelTunables=yes
ProtectKernelModules=yes
ProtectControlGroups=yes
RestrictRealtime=yes
RestrictSUIDSGID=yes
RemoveIPC=yes
PrivateTmp=yes

# File system access
ReadWritePaths=$LOG_DIR
ReadWritePaths=/var/lib/ziplock

# Network restrictions (if needed)
# PrivateNetwork=yes

# Capabilities
CapabilityBoundingSet=

# System calls
SystemCallArchitectures=native
SystemCallFilter=@system-service
SystemCallFilter=~@debug @mount @cpu-emulation @obsolete

[Install]
WantedBy=multi-user.target
EOF

    log_success "Systemd service file created: $SYSTEMD_SERVICE_DIR/ziplock.service"
}

# Generate logrotate configuration
generate_logrotate_config() {
    log_info "Creating logrotate configuration..."

    cat > "$LOGROTATE_CONFIG_DIR/ziplock" << EOF
$LOG_DIR/*.log {
    # Rotate daily
    daily

    # Keep 30 days of logs
    rotate 30

    # Compress old logs (except the most recent)
    compress
    delaycompress

    # Don't rotate if log is empty
    notifempty

    # Don't error if log file is missing
    missingok

    # Create new log file with specific permissions
    create 644 $ZIPLOCK_USER $ZIPLOCK_GROUP

    # Copy and truncate the original log file
    copytruncate

    # Execute after rotation
    postrotate
        # Send SIGHUP to ziplock process to reopen log files
        systemctl reload-or-restart ziplock.service > /dev/null 2>&1 || true

        # Alternative: Send SIGHUP to process directly
        # pkill -HUP -u $ZIPLOCK_USER ziplock > /dev/null 2>&1 || true
    endscript
}

# Separate configuration for archived logs
$LOG_DIR/*.log.* {
    # For already rotated files, rotate weekly
    weekly

    # Keep 12 weeks (3 months) of archived logs
    rotate 12

    # Compress
    compress

    # Other settings
    missingok
    notifempty
}
EOF

    log_success "Logrotate configuration created: $LOGROTATE_CONFIG_DIR/ziplock"
}

# Test logrotate configuration
test_logrotate() {
    log_info "Testing logrotate configuration..."

    if logrotate -d "$LOGROTATE_CONFIG_DIR/ziplock" >/dev/null 2>&1; then
        log_success "Logrotate configuration is valid"
    else
        log_warning "Logrotate configuration validation failed - please check manually"
    fi
}

# Setup systemd journal storage
setup_systemd_journal() {
    log_info "Configuring systemd journal storage..."

    # Ensure journald directory exists
    mkdir -p /var/log/journal

    # Configure journal storage (if not already configured)
    if ! grep -q "^Storage=" /etc/systemd/journald.conf 2>/dev/null; then
        log_info "Configuring persistent journal storage..."

        # Backup original config
        cp /etc/systemd/journald.conf /etc/systemd/journald.conf.backup.$(date +%Y%m%d_%H%M%S)

        # Add persistent storage configuration
        cat >> /etc/systemd/journald.conf << EOF

# ZipLock logging configuration
Storage=persistent
MaxFileSec=1day
MaxRetentionSec=1month
SystemMaxUse=1G
SystemKeepFree=500M
EOF

        log_success "Systemd journal configured for persistent storage"
        log_info "You may want to restart systemd-journald: systemctl restart systemd-journald"
    else
        log_info "Systemd journal storage already configured"
    fi
}

# Create application data directory
setup_app_directory() {
    log_info "Setting up application data directory..."

    APP_DIR="/var/lib/ziplock"
    mkdir -p "$APP_DIR"
    chown "$ZIPLOCK_USER:$ZIPLOCK_GROUP" "$APP_DIR"
    chmod 750 "$APP_DIR"

    log_success "Application data directory created: $APP_DIR"
}

# Enable and start service
enable_service() {
    log_info "Enabling and starting ZipLock service..."

    # Reload systemd configuration
    systemctl daemon-reload

    # Enable the service
    systemctl enable ziplock.service

    log_success "ZipLock service enabled"
    log_info "To start the service: systemctl start ziplock"
    log_info "To check status: systemctl status ziplock"
    log_info "To view logs: journalctl -u ziplock -f"
}

# Verify ZipLock binary exists
verify_binary() {
    if [[ ! -f "$ZIPLOCK_BIN" ]]; then
        log_error "ZipLock binary not found at: $ZIPLOCK_BIN"
        log_info "Please install ZipLock or specify correct path with ZIPLOCK_BIN environment variable"
        exit 1
    fi

    if [[ ! -x "$ZIPLOCK_BIN" ]]; then
        log_error "ZipLock binary is not executable: $ZIPLOCK_BIN"
        chmod +x "$ZIPLOCK_BIN"
        log_success "Made ZipLock binary executable"
    fi

    log_success "ZipLock binary verified: $ZIPLOCK_BIN"
}

# Display post-installation information
show_post_install_info() {
    log_success "ZipLock logging setup completed!"
    echo
    log_info "System Information:"
    echo "  • Service file: $SYSTEMD_SERVICE_DIR/ziplock.service"
    echo "  • Log directory: $LOG_DIR"
    echo "  • Logrotate config: $LOGROTATE_CONFIG_DIR/ziplock"
    echo "  • User/Group: $ZIPLOCK_USER:$ZIPLOCK_GROUP"
    echo "  • App directory: /var/lib/ziplock"
    echo
    log_info "Useful Commands:"
    echo "  • Start service:    systemctl start ziplock"
    echo "  • Stop service:     systemctl stop ziplock"
    echo "  • Check status:     systemctl status ziplock"
    echo "  • View logs:        journalctl -u ziplock -f"
    echo "  • View log files:   ls -la $LOG_DIR"
    echo "  • Test logrotate:   logrotate -d $LOGROTATE_CONFIG_DIR/ziplock"
    echo "  • Force logrotate:  logrotate -f $LOGROTATE_CONFIG_DIR/ziplock"
    echo
    log_info "Log Management:"
    echo "  • Logs are stored in: $LOG_DIR"
    echo "  • Logs are rotated daily, kept for 30 days"
    echo "  • Systemd journal logs: journalctl -u ziplock"
    echo "  • Application file logs: tail -f $LOG_DIR/ziplock.log"
    echo
    log_warning "Next Steps:"
    echo "  1. Review the systemd service file and modify if needed"
    echo "  2. Start the service: systemctl start ziplock"
    echo "  3. Check logs to ensure everything is working"
    echo "  4. Configure firewall if needed"
    echo "  5. Set up monitoring/alerting as required"
}

# Parse command line arguments
usage() {
    echo "Usage: $0 [OPTIONS]"
    echo
    echo "Options:"
    echo "  --user USER           System user for ZipLock (default: ziplock)"
    echo "  --group GROUP         System group for ZipLock (default: ziplock)"
    echo "  --log-dir DIR         Log directory (default: /var/log/ziplock)"
    echo "  --bin-path PATH       Path to ZipLock binary (default: /usr/bin/ziplock)"
    echo "  --skip-user-creation  Skip creating system user/group"
    echo "  --skip-journal-setup  Skip systemd journal configuration"
    echo "  --skip-service-enable Skip enabling systemd service"
    echo "  --help                Show this help message"
    echo
    echo "Environment Variables:"
    echo "  ZIPLOCK_USER          System user (same as --user)"
    echo "  ZIPLOCK_GROUP         System group (same as --group)"
    echo "  ZIPLOCK_LOG_DIR       Log directory (same as --log-dir)"
    echo "  ZIPLOCK_BIN           Binary path (same as --bin-path)"
}

# Parse arguments
SKIP_USER_CREATION=false
SKIP_JOURNAL_SETUP=false
SKIP_SERVICE_ENABLE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --user)
            ZIPLOCK_USER="$2"
            shift 2
            ;;
        --group)
            ZIPLOCK_GROUP="$2"
            shift 2
            ;;
        --log-dir)
            LOG_DIR="$2"
            shift 2
            ;;
        --bin-path)
            ZIPLOCK_BIN="$2"
            shift 2
            ;;
        --skip-user-creation)
            SKIP_USER_CREATION=true
            shift
            ;;
        --skip-journal-setup)
            SKIP_JOURNAL_SETUP=true
            shift
            ;;
        --skip-service-enable)
            SKIP_SERVICE_ENABLE=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        *)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
    esac
done

# Main execution
main() {
    log_info "Starting ZipLock logging setup..."
    log_info "User: $ZIPLOCK_USER, Group: $ZIPLOCK_GROUP"
    log_info "Log directory: $LOG_DIR"
    log_info "Binary path: $ZIPLOCK_BIN"

    check_root
    verify_binary

    if [[ "$SKIP_USER_CREATION" != "true" ]]; then
        create_user_group
    else
        log_info "Skipping user/group creation"
    fi

    setup_log_directory
    setup_app_directory
    generate_systemd_service
    generate_logrotate_config
    test_logrotate

    if [[ "$SKIP_JOURNAL_SETUP" != "true" ]]; then
        setup_systemd_journal
    else
        log_info "Skipping systemd journal setup"
    fi

    if [[ "$SKIP_SERVICE_ENABLE" != "true" ]]; then
        enable_service
    else
        log_info "Skipping service enablement"
    fi

    show_post_install_info
}

# Run main function
main "$@"

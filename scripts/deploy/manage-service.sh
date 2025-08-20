#!/bin/bash
# ZipLock Service Management Script
#
# This script provides comprehensive management of ZipLock when deployed
# as a systemd service, including installation, configuration, monitoring,
# and troubleshooting capabilities.

set -euo pipefail

# Configuration
ZIPLOCK_USER="${ZIPLOCK_USER:-ziplock}"
ZIPLOCK_GROUP="${ZIPLOCK_GROUP:-ziplock}"
LOG_DIR="${ZIPLOCK_LOG_DIR:-/var/log/ziplock}"
SYSTEMD_SERVICE_DIR="/etc/systemd/system"
SERVICE_NAME="ziplock.service"
ZIPLOCK_BIN="${ZIPLOCK_BIN:-/usr/bin/ziplock}"
CONFIG_DIR="/etc/ziplock"
DATA_DIR="/var/lib/ziplock"

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

log_debug() {
    if [[ "${DEBUG:-false}" == "true" ]]; then
        echo -e "${PURPLE}[DEBUG]${NC} $1"
    fi
}

# Check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        log_error "This operation requires root privileges (use sudo)"
        exit 1
    fi
}

# Check if service exists
service_exists() {
    systemctl list-unit-files "$SERVICE_NAME" --no-legend --no-pager | grep -q "$SERVICE_NAME"
}

# Get service status
get_service_status() {
    if service_exists; then
        systemctl is-active "$SERVICE_NAME" 2>/dev/null || echo "inactive"
    else
        echo "not-installed"
    fi
}

# Get service enabled status
get_service_enabled_status() {
    if service_exists; then
        systemctl is-enabled "$SERVICE_NAME" 2>/dev/null || echo "disabled"
    else
        echo "not-installed"
    fi
}

# Install ZipLock service
install_service() {
    log_info "Installing ZipLock service..."
    check_root

    # Check if binary exists
    if [[ ! -f "$ZIPLOCK_BIN" ]]; then
        log_error "ZipLock binary not found at: $ZIPLOCK_BIN"
        log_info "Please install ZipLock first or specify correct path"
        exit 1
    fi

    # Run the logging setup script
    if [[ -f "$(dirname "$0")/setup-logging.sh" ]]; then
        log_info "Running logging setup..."
        bash "$(dirname "$0")/setup-logging.sh"
    else
        log_warning "Logging setup script not found, creating basic service..."

        # Create directories
        mkdir -p "$LOG_DIR" "$DATA_DIR" "$CONFIG_DIR"

        # Create user if not exists
        if ! getent passwd "$ZIPLOCK_USER" >/dev/null 2>&1; then
            useradd --system --gid users \
                    --home-dir "$DATA_DIR" \
                    --shell /bin/false \
                    --comment "ZipLock Password Manager" \
                    "$ZIPLOCK_USER"
        fi

        chown "$ZIPLOCK_USER:$ZIPLOCK_GROUP" "$LOG_DIR" "$DATA_DIR"
        chmod 750 "$LOG_DIR" "$DATA_DIR"
    fi

    log_success "ZipLock service installed successfully"
}

# Start ZipLock service
start_service() {
    log_info "Starting ZipLock service..."

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_error "Service not installed. Run 'install' first."
        exit 1
    fi

    systemctl start "$SERVICE_NAME"

    # Wait a moment for startup
    sleep 2

    if [[ "$(get_service_status)" == "active" ]]; then
        log_success "ZipLock service started successfully"
    else
        log_error "Failed to start ZipLock service"
        show_service_status
        exit 1
    fi
}

# Stop ZipLock service
stop_service() {
    log_info "Stopping ZipLock service..."

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_warning "Service not installed"
        return 0
    fi

    systemctl stop "$SERVICE_NAME"

    # Wait a moment for shutdown
    sleep 2

    if [[ "$(get_service_status)" == "inactive" ]]; then
        log_success "ZipLock service stopped successfully"
    else
        log_warning "Service may still be shutting down"
    fi
}

# Restart ZipLock service
restart_service() {
    log_info "Restarting ZipLock service..."

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_error "Service not installed. Run 'install' first."
        exit 1
    fi

    systemctl restart "$SERVICE_NAME"

    # Wait a moment for restart
    sleep 3

    if [[ "$(get_service_status)" == "active" ]]; then
        log_success "ZipLock service restarted successfully"
    else
        log_error "Failed to restart ZipLock service"
        show_service_status
        exit 1
    fi
}

# Reload service configuration
reload_service() {
    log_info "Reloading ZipLock service configuration..."

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_error "Service not installed. Run 'install' first."
        exit 1
    fi

    systemctl daemon-reload

    if [[ "$(get_service_status)" == "active" ]]; then
        systemctl reload-or-restart "$SERVICE_NAME"
        log_success "ZipLock service configuration reloaded"
    else
        log_success "Configuration reloaded (service not running)"
    fi
}

# Enable ZipLock service
enable_service() {
    log_info "Enabling ZipLock service for automatic startup..."
    check_root

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_error "Service not installed. Run 'install' first."
        exit 1
    fi

    systemctl enable "$SERVICE_NAME"
    log_success "ZipLock service enabled for automatic startup"
}

# Disable ZipLock service
disable_service() {
    log_info "Disabling ZipLock service automatic startup..."
    check_root

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_warning "Service not installed"
        return 0
    fi

    systemctl disable "$SERVICE_NAME"
    log_success "ZipLock service disabled"
}

# Show service status
show_service_status() {
    local status=$(get_service_status)
    local enabled_status=$(get_service_enabled_status)

    echo "=== ZipLock Service Status ==="
    echo

    case $status in
        "active")
            log_success "Service Status: RUNNING"
            ;;
        "inactive")
            log_warning "Service Status: STOPPED"
            ;;
        "failed")
            log_error "Service Status: FAILED"
            ;;
        "not-installed")
            log_warning "Service Status: NOT INSTALLED"
            ;;
        *)
            echo "Service Status: $status"
            ;;
    esac

    case $enabled_status in
        "enabled")
            log_success "Auto-start: ENABLED"
            ;;
        "disabled")
            log_warning "Auto-start: DISABLED"
            ;;
        "not-installed")
            log_warning "Auto-start: NOT INSTALLED"
            ;;
        *)
            echo "Auto-start: $enabled_status"
            ;;
    esac

    if service_exists; then
        echo
        echo "--- Systemd Status ---"
        systemctl status "$SERVICE_NAME" --no-pager -l || true

        echo
        echo "--- Service Information ---"
        echo "Service file: $SYSTEMD_SERVICE_DIR/$SERVICE_NAME"
        echo "Binary path: $ZIPLOCK_BIN"
        echo "Log directory: $LOG_DIR"
        echo "Data directory: $DATA_DIR"
        echo "User/Group: $ZIPLOCK_USER:$ZIPLOCK_GROUP"
    fi
}

# Show service logs
show_logs() {
    local lines="${1:-50}"
    local follow="${2:-false}"

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_error "Service not installed"
        exit 1
    fi

    log_info "Showing last $lines lines of ZipLock logs..."

    if [[ "$follow" == "true" ]]; then
        log_info "Following logs (Ctrl+C to exit)..."
        journalctl -u "$SERVICE_NAME" -f -n "$lines"
    else
        journalctl -u "$SERVICE_NAME" -n "$lines" --no-pager
    fi
}

# Show file logs
show_file_logs() {
    local lines="${1:-50}"
    local follow="${2:-false}"

    if [[ ! -d "$LOG_DIR" ]]; then
        log_error "Log directory not found: $LOG_DIR"
        exit 1
    fi

    local log_files=($(find "$LOG_DIR" -name "*.log" -type f))

    if [[ ${#log_files[@]} -eq 0 ]]; then
        log_warning "No log files found in $LOG_DIR"
        return 0
    fi

    log_info "File logs in $LOG_DIR:"
    ls -la "$LOG_DIR"
    echo

    local latest_log="${log_files[0]}"
    # Find the most recent log file
    for log_file in "${log_files[@]}"; do
        if [[ "$log_file" -nt "$latest_log" ]]; then
            latest_log="$log_file"
        fi
    done

    log_info "Showing last $lines lines from: $latest_log"

    if [[ "$follow" == "true" ]]; then
        tail -f -n "$lines" "$latest_log"
    else
        tail -n "$lines" "$latest_log"
    fi
}

# Test service configuration
test_service() {
    log_info "Testing ZipLock service configuration..."

    echo "=== Configuration Test ==="
    echo

    # Check binary
    if [[ -f "$ZIPLOCK_BIN" ]] && [[ -x "$ZIPLOCK_BIN" ]]; then
        log_success "Binary exists and is executable: $ZIPLOCK_BIN"
    else
        log_error "Binary not found or not executable: $ZIPLOCK_BIN"
    fi

    # Check directories
    for dir in "$LOG_DIR" "$DATA_DIR"; do
        if [[ -d "$dir" ]]; then
            log_success "Directory exists: $dir"
            echo "  Permissions: $(stat -c '%A %U:%G' "$dir")"
        else
            log_error "Directory missing: $dir"
        fi
    done

    # Check user
    if getent passwd "$ZIPLOCK_USER" >/dev/null 2>&1; then
        log_success "User exists: $ZIPLOCK_USER"
    else
        log_error "User not found: $ZIPLOCK_USER"
    fi

    # Check service file
    if [[ -f "$SYSTEMD_SERVICE_DIR/$SERVICE_NAME" ]]; then
        log_success "Service file exists"

        # Validate service file syntax
        if systemd-analyze verify "$SYSTEMD_SERVICE_DIR/$SERVICE_NAME" 2>/dev/null; then
            log_success "Service file syntax is valid"
        else
            log_warning "Service file may have syntax issues"
        fi
    else
        log_error "Service file not found: $SYSTEMD_SERVICE_DIR/$SERVICE_NAME"
    fi

    echo
    echo "=== Runtime Test ==="

    # Test if we can start the service temporarily
    if [[ "$(get_service_status)" != "active" ]]; then
        log_info "Starting service for testing..."
        systemctl start "$SERVICE_NAME" || true
        sleep 3

        local test_status=$(get_service_status)
        if [[ "$test_status" == "active" ]]; then
            log_success "Service starts successfully"

            # Stop test service
            systemctl stop "$SERVICE_NAME"
            log_info "Stopped test service"
        else
            log_error "Service failed to start during test"
            journalctl -u "$SERVICE_NAME" -n 20 --no-pager
        fi
    else
        log_info "Service already running - skipping start test"
    fi
}

# Monitor service health
monitor_service() {
    log_info "Starting ZipLock service monitor..."
    log_info "Press Ctrl+C to stop monitoring"

    if [[ "$(get_service_status)" == "not-installed" ]]; then
        log_error "Service not installed"
        exit 1
    fi

    while true; do
        clear
        echo "=== ZipLock Service Monitor ($(date)) ==="
        echo

        show_service_status

        echo
        echo "--- Recent Logs ---"
        journalctl -u "$SERVICE_NAME" -n 10 --no-pager || true

        echo
        echo "--- System Resources ---"
        if pgrep -f "$ZIPLOCK_BIN" >/dev/null; then
            local pid=$(pgrep -f "$ZIPLOCK_BIN")
            echo "Process ID: $pid"
            ps -p "$pid" -o pid,cpu,mem,time,cmd 2>/dev/null || echo "Process info unavailable"
        else
            echo "ZipLock process not found"
        fi

        echo
        echo "--- Log Files ---"
        if [[ -d "$LOG_DIR" ]]; then
            ls -lah "$LOG_DIR" | head -10
        else
            echo "Log directory not found: $LOG_DIR"
        fi

        echo
        echo "Refreshing in 5 seconds... (Ctrl+C to exit)"
        sleep 5
    done
}

# Uninstall service
uninstall_service() {
    log_warning "This will completely remove ZipLock service and data!"
    read -p "Are you sure? (y/N): " -n 1 -r
    echo

    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        log_info "Uninstall cancelled"
        return 0
    fi

    check_root
    log_info "Uninstalling ZipLock service..."

    # Stop and disable service
    if service_exists; then
        systemctl stop "$SERVICE_NAME" || true
        systemctl disable "$SERVICE_NAME" || true
        rm -f "$SYSTEMD_SERVICE_DIR/$SERVICE_NAME"
        systemctl daemon-reload
        log_success "Service removed"
    fi

    # Remove logrotate config
    if [[ -f "/etc/logrotate.d/ziplock" ]]; then
        rm -f "/etc/logrotate.d/ziplock"
        log_success "Logrotate configuration removed"
    fi

    # Optionally remove data and logs
    read -p "Remove log and data directories? (y/N): " -n 1 -r
    echo

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        rm -rf "$LOG_DIR" "$DATA_DIR" "$CONFIG_DIR"
        log_success "Data directories removed"

        # Remove user
        if getent passwd "$ZIPLOCK_USER" >/dev/null 2>&1; then
            userdel "$ZIPLOCK_USER" || true
            log_success "User removed"
        fi
    fi

    log_success "ZipLock service uninstalled"
}

# Display usage information
usage() {
    echo "Usage: $0 COMMAND [OPTIONS]"
    echo
    echo "Comprehensive ZipLock service management"
    echo
    echo "Commands:"
    echo "  install        Install ZipLock as a systemd service"
    echo "  start          Start the ZipLock service"
    echo "  stop           Stop the ZipLock service"
    echo "  restart        Restart the ZipLock service"
    echo "  reload         Reload service configuration"
    echo "  enable         Enable automatic startup"
    echo "  disable        Disable automatic startup"
    echo "  status         Show detailed service status"
    echo "  logs [LINES]   Show systemd logs (default: 50 lines)"
    echo "  logs-follow    Follow systemd logs in real-time"
    echo "  file-logs      Show file-based logs"
    echo "  file-logs-follow Follow file-based logs in real-time"
    echo "  test           Test service configuration"
    echo "  monitor        Monitor service health (interactive)"
    echo "  uninstall      Remove service and data"
    echo "  help           Show this help message"
    echo
    echo "Options:"
    echo "  --user USER    System user (default: ziplock)"
    echo "  --group GROUP  System group (default: ziplock)"
    echo "  --log-dir DIR  Log directory (default: /var/log/ziplock)"
    echo "  --debug        Enable debug output"
    echo
    echo "Environment Variables:"
    echo "  ZIPLOCK_USER   System user"
    echo "  ZIPLOCK_GROUP  System group"
    echo "  ZIPLOCK_LOG_DIR Log directory"
    echo "  ZIPLOCK_BIN    Path to ZipLock binary"
    echo "  DEBUG          Enable debug output (set to 'true')"
    echo
    echo "Examples:"
    echo "  $0 install                    # Install service with defaults"
    echo "  $0 start                      # Start the service"
    echo "  $0 logs 100                   # Show last 100 log lines"
    echo "  $0 --user myuser install      # Install with custom user"
    echo "  $0 monitor                    # Start interactive monitoring"
}

# Parse command line arguments
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
        --debug)
            export DEBUG=true
            shift
            ;;
        --help)
            usage
            exit 0
            ;;
        -*)
            log_error "Unknown option: $1"
            usage
            exit 1
            ;;
        *)
            break
            ;;
    esac
done

# Main command processing
if [[ $# -eq 0 ]]; then
    usage
    exit 1
fi

COMMAND="$1"
shift

case "$COMMAND" in
    install)
        install_service
        ;;
    start)
        start_service
        ;;
    stop)
        stop_service
        ;;
    restart)
        restart_service
        ;;
    reload)
        reload_service
        ;;
    enable)
        enable_service
        ;;
    disable)
        disable_service
        ;;
    status)
        show_service_status
        ;;
    logs)
        lines="${1:-50}"
        show_logs "$lines" false
        ;;
    logs-follow)
        lines="${1:-50}"
        show_logs "$lines" true
        ;;
    file-logs)
        lines="${1:-50}"
        show_file_logs "$lines" false
        ;;
    file-logs-follow)
        lines="${1:-50}"
        show_file_logs "$lines" true
        ;;
    test)
        test_service
        ;;
    monitor)
        monitor_service
        ;;
    uninstall)
        uninstall_service
        ;;
    help)
        usage
        ;;
    *)
        log_error "Unknown command: $COMMAND"
        echo
        usage
        exit 1
        ;;
esac

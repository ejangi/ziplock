#!/bin/bash

# WiX Validation Script
# Validates WiX source files for common issues without building MSI

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
WXS_FILE="$PROJECT_ROOT/packaging/windows/installer/ziplock.wxs"

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
    echo -e "${CYAN}[STEP]${NC} $1"
}

# Validation functions
validate_xml_syntax() {
    log_step "Validating XML syntax..."

    if ! command -v xmllint &> /dev/null; then
        log_warning "xmllint not found, skipping XML syntax validation"
        return 0
    fi

    if xmllint --noout "$WXS_FILE" 2>/dev/null; then
        log_success "XML syntax is valid"
    else
        log_error "XML syntax validation failed"
        return 1
    fi
}

validate_wix_namespace() {
    log_step "Checking WiX v4 namespace..."

    if grep -q 'xmlns="http://wixtoolset.org/schemas/v4/wxs"' "$WXS_FILE"; then
        log_success "WiX v4 namespace found"
    else
        log_error "WiX v4 namespace not found"
        return 1
    fi

    if grep -q 'xmlns:ui="http://wixtoolset.org/schemas/v4/wxs/ui"' "$WXS_FILE"; then
        log_success "WiX UI namespace found"
    else
        log_warning "WiX UI namespace not found (may not be needed)"
    fi
}

validate_package_element() {
    log_step "Checking Package element structure..."

    if grep -q '<Package ' "$WXS_FILE"; then
        log_success "Package element found (WiX v4 format)"
    else
        if grep -q '<Product ' "$WXS_FILE"; then
            log_error "Product element found - this is WiX v3 syntax, should be Package for v4"
            return 1
        else
            log_error "Neither Package nor Product element found"
            return 1
        fi
    fi
}

validate_ui_elements() {
    log_step "Checking UI element syntax..."

    # Check for old UIRef syntax
    if grep -q '<UIRef ' "$WXS_FILE"; then
        log_warning "Old UIRef syntax found - consider using WiX v4 ui: namespace"
    fi

    # Check for new ui:WixUI syntax
    if grep -q '<ui:WixUI ' "$WXS_FILE"; then
        log_success "WiX v4 UI syntax found"
    fi

    # Check for common UI dialog sets
    if grep -q 'WixUI_InstallDir' "$WXS_FILE"; then
        log_success "WixUI_InstallDir dialog set referenced"
    fi
}

validate_component_structure() {
    log_step "Checking Component structure..."

    # Check for Guid="*" which is deprecated in v4
    if grep -q 'Guid="\*"' "$WXS_FILE"; then
        log_warning "Guid=\"*\" found - this is deprecated in WiX v4"
    fi

    # Check for KeyPath="yes" which is automatic in v4
    if grep -q 'KeyPath="yes"' "$WXS_FILE"; then
        log_warning "KeyPath=\"yes\" found - this is automatic in WiX v4"
    fi

    # Check for proper Component elements
    component_count=$(grep -c '<Component ' "$WXS_FILE" || true)
    if [ "$component_count" -gt 0 ]; then
        log_success "Found $component_count Component elements"
    else
        log_error "No Component elements found"
        return 1
    fi
}

validate_directory_structure() {
    log_step "Checking Directory structure..."

    # Check for StandardDirectory usage (v4 preferred)
    if grep -q '<StandardDirectory ' "$WXS_FILE"; then
        log_success "StandardDirectory elements found (WiX v4 preferred)"
    else
        if grep -q '<Directory Id="TARGETDIR"' "$WXS_FILE"; then
            log_warning "TARGETDIR found - consider using StandardDirectory for WiX v4"
        fi
    fi
}

validate_file_references() {
    log_step "Checking file references..."

    # Check for Source attribute in File elements
    if grep -q 'Source=.*SourceDir' "$WXS_FILE"; then
        log_success "SourceDir variable references found"
    else
        log_warning "No SourceDir variable references found"
    fi

    # Check for version binding
    if grep -q 'bind.FileVersion' "$WXS_FILE"; then
        log_success "File version binding found"
    else
        log_info "No file version binding found (optional)"
    fi
}

validate_properties() {
    log_step "Checking Properties..."

    # Check for required properties
    if grep -q 'WIXUI_INSTALLDIR' "$WXS_FILE"; then
        log_success "WIXUI_INSTALLDIR property found"
    else
        log_warning "WIXUI_INSTALLDIR property not found (needed for InstallDir dialog)"
    fi

    # Check for ARP (Add/Remove Programs) properties
    arp_count=$(grep -c 'ARP' "$WXS_FILE" || true)
    if [ "$arp_count" -gt 0 ]; then
        log_success "Found $arp_count Add/Remove Programs properties"
    else
        log_info "No ARP properties found (optional but recommended)"
    fi
}

validate_features() {
    log_step "Checking Feature structure..."

    feature_count=$(grep -c '<Feature ' "$WXS_FILE" || true)
    if [ "$feature_count" -gt 0 ]; then
        log_success "Found $feature_count Feature elements"
    else
        log_error "No Feature elements found"
        return 1
    fi

    # Check for ComponentRef elements
    compref_count=$(grep -c '<ComponentRef ' "$WXS_FILE" || true)
    if [ "$compref_count" -gt 0 ]; then
        log_success "Found $compref_count ComponentRef elements"
    else
        log_warning "No ComponentRef elements found"
    fi
}

check_deprecated_elements() {
    log_step "Checking for deprecated elements..."

    # MediaTemplate is replaced with Media in v4
    if grep -q '<MediaTemplate ' "$WXS_FILE"; then
        log_warning "MediaTemplate found - consider using Media element for WiX v4"
    fi

    # ProgId/Extension are handled differently in v4
    if grep -q '<ProgId ' "$WXS_FILE"; then
        log_warning "ProgId element found - file associations are handled differently in WiX v4"
    fi
}

show_summary() {
    echo
    log_info "WiX Validation Summary"
    log_info "====================="
    log_info "WXS File: $WXS_FILE"

    if [ -f "$WXS_FILE" ]; then
        file_size=$(wc -c < "$WXS_FILE")
        line_count=$(wc -l < "$WXS_FILE")
        log_info "File size: $file_size bytes"
        log_info "Line count: $line_count lines"
    fi
}

# Main execution
main() {
    log_info "WiX Source File Validation"
    log_info "=========================="

    if [ ! -f "$WXS_FILE" ]; then
        log_error "WiX source file not found: $WXS_FILE"
        exit 1
    fi

    log_info "Validating: $WXS_FILE"
    echo

    # Run all validations
    validation_failed=false

    validate_xml_syntax || validation_failed=true
    validate_wix_namespace || validation_failed=true
    validate_package_element || validation_failed=true
    validate_ui_elements
    validate_component_structure
    validate_directory_structure
    validate_file_references
    validate_properties
    validate_features || validation_failed=true
    check_deprecated_elements

    show_summary

    if [ "$validation_failed" = true ]; then
        echo
        log_error "Validation completed with errors"
        exit 1
    else
        echo
        log_success "Validation completed successfully!"
        log_info "The WiX file appears to be compatible with WiX v4"
    fi
}

# Check if running directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi

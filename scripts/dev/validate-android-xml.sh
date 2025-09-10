#!/bin/bash

# ZipLock Android XML Validation Script
# Validates Android backup and data extraction XML files for common lint issues

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ANDROID_DIR="$PROJECT_ROOT/apps/mobile/android"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

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

log_header() {
    echo -e "${BOLD}${BLUE}=== $1 ===${NC}"
}

# Show usage
show_usage() {
    cat << EOF
Usage: $0 [options]

Validate Android backup and data extraction XML files for common lint issues.

This script checks for the FullBackupContent lint errors that were causing
the CI build to fail. It validates XML syntax and backup rule compliance.

OPTIONS:
    -h, --help     Show this help message
    -v, --verbose  Show detailed validation information

EXAMPLES:
    $0             # Run basic validation
    $0 -v          # Run with verbose output

This script can run without a full Android SDK installation.
EOF
}

# Check if xmllint is available
check_xmllint() {
    if ! command -v xmllint &> /dev/null; then
        log_warning "xmllint not found - XML syntax validation will be skipped"
        log_info "Install libxml2-utils (Ubuntu/Debian) or libxml2 (macOS) for syntax validation"
        return 1
    fi
    return 0
}

# Validate XML syntax
validate_xml_syntax() {
    local xml_file="$1"
    local filename=$(basename "$xml_file")

    if check_xmllint; then
        if xmllint --noout "$xml_file" 2>/dev/null; then
            log_success "✅ $filename - Valid XML syntax"
            return 0
        else
            log_error "❌ $filename - Invalid XML syntax:"
            xmllint "$xml_file" || true
            return 1
        fi
    else
        log_info "⏭️  $filename - XML syntax check skipped (xmllint not available)"
        return 0
    fi
}

# Analyze backup rules for common lint issues
analyze_backup_rules() {
    local xml_file="$1"
    local filename=$(basename "$xml_file")
    local issues=0
    local verbose="${VERBOSE:-false}"

    log_info "🔍 Analyzing $filename for backup rule compliance..."

    # Read file content
    local content
    if ! content=$(cat "$xml_file"); then
        log_error "❌ Could not read $xml_file"
        return 1
    fi

    # Count include and exclude rules by domain
    local includes_file=$(echo "$content" | grep -c '<include.*domain="file"' 2>/dev/null) || includes_file=0
    local includes_sharedpref=$(echo "$content" | grep -c '<include.*domain="sharedpref"' 2>/dev/null) || includes_sharedpref=0
    local includes_database=$(echo "$content" | grep -c '<include.*domain="database"' 2>/dev/null) || includes_database=0

    local excludes_file=$(echo "$content" | grep -c '<exclude.*domain="file"' 2>/dev/null) || excludes_file=0
    local excludes_sharedpref=$(echo "$content" | grep -c '<exclude.*domain="sharedpref"' 2>/dev/null) || excludes_sharedpref=0
    local excludes_database=$(echo "$content" | grep -c '<exclude.*domain="database"' 2>/dev/null) || excludes_database=0

    if [ "$verbose" = "true" ]; then
        log_info "  📊 Include rules:"
        log_info "    - file domain: $includes_file"
        log_info "    - sharedpref domain: $includes_sharedpref"
        log_info "    - database domain: $includes_database"
        log_info "  📊 Exclude rules:"
        log_info "    - file domain: $excludes_file"
        log_info "    - sharedpref domain: $excludes_sharedpref"
        log_info "    - database domain: $excludes_database"
    fi

    # Check for FullBackupContent violations

    # Issue 1: Excluding from sharedpref domain without including it
    if [ "$excludes_sharedpref" -gt 0 ] && [ "$includes_sharedpref" -eq 0 ]; then
        log_error "❌ Found exclude rules for 'sharedpref' domain without any include rules"
        log_error "   This causes: 'is not in an included path [FullBackupContent]'"
        issues=$((issues + 1))
    fi

    # Issue 2: Excluding from database domain without including it
    if [ "$excludes_database" -gt 0 ] && [ "$includes_database" -eq 0 ]; then
        log_error "❌ Found exclude rules for 'database' domain without any include rules"
        log_error "   This causes: 'is not in an included path [FullBackupContent]'"
        issues=$((issues + 1))
    fi

    # Issue 3: Check specific file paths being excluded without being included
    if [ "$excludes_file" -gt 0 ]; then
        # Extract exclude paths
        local exclude_paths
        exclude_paths=$(echo "$content" | grep '<exclude.*domain="file"' 2>/dev/null | sed -n 's/.*path="\([^"]*\)".*/\1/p' 2>/dev/null) || exclude_paths=""

        if [ -n "$exclude_paths" ]; then
            while IFS= read -r exclude_path; do
                if [ -n "$exclude_path" ]; then
                    # Check if this path is included
                    local path_included=false

                    # Look for explicit include of this path
                    if echo "$content" | grep -q "<include.*domain=\"file\".*path=\"$exclude_path\""; then
                        path_included=true
                    fi

                    # Check if there's a wildcard include that would cover this path
                    if echo "$content" | grep -q '<include.*domain="file".*path=".*"' && [ "$exclude_path" != "." ]; then
                        # There's a wildcard include, so specific excludes are valid
                        path_included=true
                    fi

                    if [ "$path_included" = "false" ] && [ "$includes_file" -eq 0 ]; then
                        log_error "❌ Found exclude rule for file path '$exclude_path' without any file includes"
                        log_error "   This causes: '$exclude_path is not in an included path [FullBackupContent]'"
                        issues=$((issues + 1))
                    fi
                fi
            done <<< "$exclude_paths"
        fi
    fi

    # Show positive findings
    if [ "$issues" -eq 0 ]; then
        log_success "✅ $filename - No backup rule violations found"

        # Show security-positive findings
        if [ "$includes_file" -gt 0 ] && [ "$includes_sharedpref" -eq 0 ] && [ "$includes_database" -eq 0 ]; then
            log_success "🔒 Security: Only specific file paths are included in backup"
        fi

        if [ "$excludes_sharedpref" -eq 0 ] && [ "$excludes_database" -eq 0 ]; then
            log_success "🔒 Security: No sensitive domains (sharedpref/database) in backup scope"
        fi
    else
        log_error "❌ $filename - Found $issues backup rule violation(s)"
    fi

    return $issues
}

# Validate all XML files
validate_all_files() {
    local verbose="${1:-false}"
    local total_issues=0

    log_header "Validating Android XML Configuration Files"

    local xml_files=(
        "$ANDROID_DIR/app/src/main/res/xml/backup_rules.xml"
        "$ANDROID_DIR/app/src/main/res/xml/data_extraction_rules.xml"
    )

    for xml_file in "${xml_files[@]}"; do
        local filename=$(basename "$xml_file")

        if [ ! -f "$xml_file" ]; then
            log_warning "⚠️  XML file not found: $xml_file"
            continue
        fi

        echo
        log_info "🔍 Validating $filename..."

        # 1. XML syntax validation
        if ! validate_xml_syntax "$xml_file"; then
            total_issues=$((total_issues + 1))
            continue
        fi

        # 2. Backup rules analysis
        local file_issues=0
        if [[ "$filename" == *"backup"* ]] || [[ "$filename" == *"extraction"* ]]; then
            analyze_backup_rules "$xml_file"
            file_issues=$?
            total_issues=$((total_issues + file_issues))
        fi

        # 3. Show file content if verbose
        if [ "$verbose" = "true" ]; then
            log_info "📄 Content of $filename:"
            echo "----------------------------------------"
            cat "$xml_file"
            echo "----------------------------------------"
        fi
    done

    echo
    log_header "Validation Summary"

    if [ "$total_issues" -eq 0 ]; then
        log_success "🎉 All XML files passed validation!"
        log_success "✅ No FullBackupContent lint errors detected"
        log_info "💡 The Android build should now pass the lintVitalRelease task"
    else
        log_error "❌ Found $total_issues issue(s) across XML files"
        log_error "🚫 The Android build will likely fail with FullBackupContent errors"
        log_info "💡 Fix the issues above and run this script again"
        return 1
    fi
}

# Show Android lint context
show_lint_context() {
    log_header "Android Lint Context"
    cat << EOF
The FullBackupContent lint rule ensures that backup configuration files
are valid according to Android's backup framework rules:

1. You can only exclude paths that would otherwise be included
2. If you exclude from a domain (sharedpref, database), that domain must be included
3. Exclude rules must reference paths that are within included paths

Common violations that cause CI failures:
- <exclude domain="sharedpref" path="." /> without any sharedpref includes
- <exclude domain="database" path="." /> without any database includes
- <exclude domain="file" path="cache/" /> without including file paths

The fix is usually to:
- Remove exclude rules for domains you don't want to backup at all
- Only exclude specific paths within included domains
- Use a whitelist approach (only include what you want) vs blacklist

For security apps like password managers, it's better to only include
specific non-sensitive paths rather than excluding sensitive ones.
EOF
}

# Main function
main() {
    local verbose=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -h|--help)
                show_usage
                exit 0
                ;;
            -v|--verbose)
                verbose=true
                shift
                ;;
            --lint-context)
                show_lint_context
                exit 0
                ;;
            *)
                log_error "Unknown option: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    # Check if Android directory exists
    if [ ! -d "$ANDROID_DIR" ]; then
        log_error "Android project directory not found: $ANDROID_DIR"
        log_error "Make sure you're running this from the ZipLock project root"
        exit 1
    fi

    # Set verbose flag for functions
    export VERBOSE="$verbose"

    # Run validation
    if validate_all_files "$verbose"; then
        log_info "🔧 To test the full Android lint locally, run:"
        log_info "   ./scripts/dev/test-android-lint.sh quick"
        exit 0
    else
        log_info "🔧 After fixing the issues, you can test with:"
        log_info "   ./scripts/dev/validate-android-xml.sh"
        log_info "   ./scripts/dev/test-android-lint.sh quick"
        exit 1
    fi
}

# Run main function
main "$@"

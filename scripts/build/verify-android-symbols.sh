#!/bin/bash
set -euo pipefail

# ZipLock Android Symbol Verification Script
# Verifies that Android native libraries contain the expected symbols and are properly built

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
ANDROID_LIBS_DIR="$PROJECT_ROOT/target/android"
OUTPUT_DIR="$PROJECT_ROOT/target/symbol-verification"

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

# Show usage
show_usage() {
    cat << EOF
Usage: $0 <command> [options]

Verify Android native library symbols and build quality.

COMMANDS:
    verify         Verify all libraries have expected symbols
    analyze        Detailed symbol analysis
    export         Export symbol information
    compare        Compare symbols between architectures
    all            Run all verification steps

OPTIONS:
    -v, --verbose    Enable verbose output
    -o, --output     Output directory (default: target/symbol-verification)
    --strict         Fail on any warnings

EXAMPLES:
    $0 verify                   # Basic symbol verification
    $0 analyze -v               # Detailed analysis with verbose output
    $0 export                   # Export symbol tables
    $0 all --strict             # Complete verification with strict checking

EOF
}

# Check dependencies
check_dependencies() {
    local missing_tools=()

    for tool in readelf nm objdump strings file; do
        if ! command -v "$tool" &> /dev/null; then
            missing_tools+=("$tool")
        fi
    done

    if [ ${#missing_tools[@]} -ne 0 ]; then
        log_error "Missing required tools: ${missing_tools[*]}"
        log_info "Install with: sudo apt-get install binutils file"
        exit 1
    fi
}

# Initialize environment
init_environment() {
    if [ ! -d "$ANDROID_LIBS_DIR" ]; then
        log_error "Android libraries not found at $ANDROID_LIBS_DIR"
        log_info "Run './scripts/build/build-android-docker.sh build' first"
        exit 1
    fi

    mkdir -p "$OUTPUT_DIR"
}

# Expected symbols for ZipLock FFI
EXPECTED_SYMBOLS=(
    "ziplock_create_repository"
    "ziplock_open_repository"
    "ziplock_save_repository"
    "ziplock_close_repository"
    "ziplock_add_credential"
    "ziplock_get_credential"
    "ziplock_list_credentials"
    "ziplock_delete_credential"
    "ziplock_free_string"
    "ziplock_free_credential_list"
)

# Verify basic symbols
verify_symbols() {
    log_info "Verifying Android library symbols..."

    local verification_report="$OUTPUT_DIR/symbol-verification.txt"
    local failed_checks=0
    local total_checks=0

    echo "=== ZipLock Android Symbol Verification ===" > "$verification_report"
    echo "Timestamp: $(date)" >> "$verification_report"
    echo >> "$verification_report"

    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        echo "=== $arch ===" >> "$verification_report"

        if [ ! -f "$lib_path" ]; then
            echo "✗ Library file not found" >> "$verification_report"
            log_warning "Library not found for $arch"
            ((failed_checks++))
            ((total_checks++))
            echo >> "$verification_report"
            continue
        fi

        echo "✓ Library file exists" >> "$verification_report"
        ((total_checks++))

        # Get all symbols
        local all_symbols
        all_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep -E "FUNC|OBJECT" | grep -v "UND" || true)

        # Check for expected symbols
        local found_symbols=0
        echo >> "$verification_report"
        echo "Expected ZipLock symbols:" >> "$verification_report"

        for symbol in "${EXPECTED_SYMBOLS[@]}"; do
            ((total_checks++))
            if echo "$all_symbols" | grep -q "$symbol"; then
                echo "  ✓ $symbol" >> "$verification_report"
                ((found_symbols++))
            else
                echo "  ✗ $symbol (MISSING)" >> "$verification_report"
                ((failed_checks++))
            fi
        done

        echo >> "$verification_report"
        echo "Symbol summary: $found_symbols/${#EXPECTED_SYMBOLS[@]} expected symbols found" >> "$verification_report"

        # Check for basic C library symbols
        echo >> "$verification_report"
        echo "Essential dependencies:" >> "$verification_report"

        local essential_deps=("malloc" "free" "strlen" "memcpy")
        for dep in "${essential_deps[@]}"; do
            ((total_checks++))
            if readelf -s "$lib_path" 2>/dev/null | grep -q "$dep"; then
                echo "  ✓ $dep available" >> "$verification_report"
            else
                echo "  ⚠ $dep not found" >> "$verification_report"
                log_warning "$dep symbol not found in $arch"
            fi
        done

        # Check symbol visibility
        echo >> "$verification_report"
        echo "Symbol visibility:" >> "$verification_report"

        local global_symbols
        global_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep " GLOBAL " | wc -l)
        local local_symbols
        local_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep " LOCAL " | wc -l)

        echo "  Global symbols: $global_symbols" >> "$verification_report"
        echo "  Local symbols: $local_symbols" >> "$verification_report"

        if [ "$global_symbols" -gt 0 ] && [ "$global_symbols" -lt 1000 ]; then
            echo "  ✓ Reasonable number of global symbols" >> "$verification_report"
        else
            echo "  ⚠ Unusual number of global symbols" >> "$verification_report"
        fi

        echo >> "$verification_report"
    done

    # Summary
    echo "=== Verification Summary ===" >> "$verification_report"
    echo "Total checks: $total_checks" >> "$verification_report"
    echo "Failed checks: $failed_checks" >> "$verification_report"
    echo "Success rate: $(( (total_checks - failed_checks) * 100 / total_checks ))%" >> "$verification_report"

    if [ "$failed_checks" -eq 0 ]; then
        log_success "All symbol verification checks passed"
    else
        log_warning "$failed_checks out of $total_checks checks failed"
    fi

    log_info "Verification report saved to: $verification_report"
    return $failed_checks
}

# Detailed symbol analysis
analyze_symbols() {
    log_info "Performing detailed symbol analysis..."

    local analysis_report="$OUTPUT_DIR/symbol-analysis.txt"

    echo "=== ZipLock Android Detailed Symbol Analysis ===" > "$analysis_report"
    echo "Timestamp: $(date)" >> "$analysis_report"
    echo >> "$analysis_report"

    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        if [ ! -f "$lib_path" ]; then
            continue
        fi

        echo "=== $arch Analysis ===" >> "$analysis_report"

        # File information
        echo "File Information:" >> "$analysis_report"
        file "$lib_path" | sed 's/^/  /' >> "$analysis_report"
        echo "  Size: $(stat -c%s "$lib_path") bytes" >> "$analysis_report"
        echo >> "$analysis_report"

        # ELF header information
        echo "ELF Header:" >> "$analysis_report"
        readelf -h "$lib_path" 2>/dev/null | sed 's/^/  /' >> "$analysis_report"
        echo >> "$analysis_report"

        # Section information
        echo "Sections:" >> "$analysis_report"
        readelf -S "$lib_path" 2>/dev/null | grep -E "\.(text|data|rodata|bss)" | sed 's/^/  /' >> "$analysis_report"
        echo >> "$analysis_report"

        # Dynamic dependencies
        echo "Dynamic Dependencies:" >> "$analysis_report"
        readelf -d "$lib_path" 2>/dev/null | grep "NEEDED" | sed 's/^/  /' >> "$analysis_report"
        echo >> "$analysis_report"

        # Symbol statistics
        echo "Symbol Statistics:" >> "$analysis_report"

        local total_symbols
        total_symbols=$(readelf -s "$lib_path" 2>/dev/null | wc -l)
        echo "  Total symbol table entries: $total_symbols" >> "$analysis_report"

        local defined_symbols
        defined_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep -v "UND" | grep -c "FUNC\|OBJECT" || echo 0)
        echo "  Defined symbols: $defined_symbols" >> "$analysis_report"

        local undefined_symbols
        undefined_symbols=$(readelf -s "$lib_path" 2>/dev/null | grep -c "UND" || echo 0)
        echo "  Undefined symbols: $undefined_symbols" >> "$analysis_report"

        # ZipLock specific symbols
        echo >> "$analysis_report"
        echo "ZipLock Symbols:" >> "$analysis_report"
        readelf -s "$lib_path" 2>/dev/null | grep "ziplock_" | sed 's/^/  /' >> "$analysis_report" || echo "  No ziplock symbols found" >> "$analysis_report"

        # Exported functions
        echo >> "$analysis_report"
        echo "Exported Functions (sample):" >> "$analysis_report"
        readelf -s "$lib_path" 2>/dev/null | grep " FUNC " | grep " GLOBAL " | head -10 | sed 's/^/  /' >> "$analysis_report"

        # String analysis
        echo >> "$analysis_report"
        echo "String Constants (sample):" >> "$analysis_report"
        strings "$lib_path" 2>/dev/null | grep -E "(error|warning|debug|ziplock)" | head -5 | sed 's/^/  /' >> "$analysis_report" || echo "  No relevant strings found" >> "$analysis_report"

        echo >> "$analysis_report"
    done

    log_success "Detailed symbol analysis completed"
    log_info "Analysis report saved to: $analysis_report"
}

# Export symbol information
export_symbols() {
    log_info "Exporting symbol information..."

    local export_dir="$OUTPUT_DIR/exports"
    mkdir -p "$export_dir"

    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")

    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"

        if [ ! -f "$lib_path" ]; then
            continue
        fi

        log_info "Exporting symbols for $arch..."

        # Export symbol table
        readelf -s "$lib_path" > "$export_dir/${arch}-symbols.txt" 2>/dev/null || true

        # Export dynamic symbols
        readelf -s --dyn-syms "$lib_path" > "$export_dir/${arch}-dynamic-symbols.txt" 2>/dev/null || true

        # Export strings
        strings "$lib_path" > "$export_dir/${arch}-strings.txt" 2>/dev/null || true

        # Export disassembly of text section (first 100 lines)
        objdump -d "$lib_path" 2>/dev/null | head -100 > "$export_dir/${arch}-disasm-sample.txt" || true

        # Create symbol summary
        local summary_file="$export_dir/${arch}-summary.txt"
        echo "=== $arch Symbol Summary ===" > "$summary_file"
        echo "Generated: $(date)" >> "$summary_file"
        echo >> "$summary_file"

        echo "Library: $lib_path" >> "$summary_file"
        echo "Size: $(stat -c%s "$lib_path") bytes" >> "$summary_file"
        echo >> "$summary_file"

        echo "Symbol Counts:" >> "$summary_file"
        echo "  Total: $(readelf -s "$lib_path" 2>/dev/null | wc -l)" >> "$summary_file"
        echo "  Functions: $(readelf -s "$lib_path" 2>/dev/null | grep -c " FUNC " || echo 0)" >> "$summary_file"
        echo "  Objects: $(readelf -s "$lib_path" 2>/dev/null | grep -c " OBJECT " || echo 0)" >> "$summary_file"
        echo "  Global: $(readelf -s "$lib_path" 2>/dev/null | grep -c " GLOBAL " || echo 0)" >> "$summary_file"
        echo "  Local: $(readelf -s "$lib_path" 2>/dev/null | grep -c " LOCAL " || echo 0)" >> "$summary_file"
        echo >> "$summary_file"

        echo "ZipLock Functions:" >> "$summary_file"
        readelf -s "$lib_path" 2>/dev/null | grep "ziplock_" | awk '{print "  " $8}' >> "$summary_file" || echo "  None found" >> "$summary_file"
    done

    log_success "Symbol information exported to: $export_dir"
}

# Compare symbols between architectures
compare_symbols() {
    log_info "Comparing symbols between architectures..."

    local comparison_report="$OUTPUT_DIR/symbol-comparison.txt"

    echo "=== ZipLock Android Symbol Comparison ===" > "$comparison_report"
    echo "Timestamp: $(date)" >> "$comparison_report"
    echo >> "$comparison_report"

    local architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")
    local arch_files=()

    # Check which libraries exist
    for arch in "${architectures[@]}"; do
        local lib_path="$ANDROID_LIBS_DIR/jniLibs/$arch/libziplock_shared.so"
        if [ -f "$lib_path" ]; then
            arch_files+=("$arch:$lib_path")
        fi
    done

    if [ ${#arch_files[@]} -lt 2 ]; then
        echo "Insufficient libraries for comparison (need at least 2)" >> "$comparison_report"
        log_warning "Need at least 2 libraries for comparison"
        return
    fi

    # Compare ZipLock symbols across architectures
    echo "ZipLock Symbol Consistency Check:" >> "$comparison_report"
    echo >> "$comparison_report"

    # Create temporary files for each architecture's ziplock symbols
    local temp_dir
    temp_dir=$(mktemp -d)

    for entry in "${arch_files[@]}"; do
        local arch="${entry%%:*}"
        local lib_path="${entry##*:}"

        readelf -s "$lib_path" 2>/dev/null | grep "ziplock_" | awk '{print $8}' | sort > "$temp_dir/${arch}.symbols"
    done

    # Compare first two architectures as baseline
    local base_arch="${arch_files[0]%%:*}"
    local base_symbols="$temp_dir/${base_arch}.symbols"

    echo "Using $base_arch as baseline" >> "$comparison_report"
    echo >> "$comparison_report"

    for entry in "${arch_files[@]:1}"; do
        local arch="${entry%%:*}"
        local arch_symbols="$temp_dir/${arch}.symbols"

        echo "Comparing $base_arch vs $arch:" >> "$comparison_report"

        # Find common symbols
        local common_symbols
        common_symbols=$(comm -12 "$base_symbols" "$arch_symbols" | wc -l)
        echo "  Common symbols: $common_symbols" >> "$comparison_report"

        # Find symbols only in base
        local base_only
        base_only=$(comm -23 "$base_symbols" "$arch_symbols")
        if [ -n "$base_only" ]; then
            echo "  Only in $base_arch:" >> "$comparison_report"
            echo "$base_only" | sed 's/^/    /' >> "$comparison_report"
        fi

        # Find symbols only in current arch
        local arch_only
        arch_only=$(comm -13 "$base_symbols" "$arch_symbols")
        if [ -n "$arch_only" ]; then
            echo "  Only in $arch:" >> "$comparison_report"
            echo "$arch_only" | sed 's/^/    /' >> "$comparison_report"
        fi

        if [ -z "$base_only" ] && [ -z "$arch_only" ]; then
            echo "  ✓ Perfect symbol match" >> "$comparison_report"
        fi

        echo >> "$comparison_report"
    done

    # Compare library sizes
    echo "Library Size Comparison:" >> "$comparison_report"
    echo "| Architecture | Size (bytes) | Size (MB) |" >> "$comparison_report"
    echo "|--------------|--------------|-----------|" >> "$comparison_report"

    for entry in "${arch_files[@]}"; do
        local arch="${entry%%:*}"
        local lib_path="${entry##*:}"

        local size_bytes
        size_bytes=$(stat -c%s "$lib_path")
        local size_mb
        size_mb=$(echo "scale=2; $size_bytes / 1024 / 1024" | bc -l)

        echo "| $arch | $size_bytes | ${size_mb} MB |" >> "$comparison_report"
    done

    # Cleanup
    rm -rf "$temp_dir"

    log_success "Symbol comparison completed"
    log_info "Comparison report saved to: $comparison_report"
}

# Run all verification steps
run_all_verification() {
    log_info "Running complete symbol verification..."

    local start_time
    start_time=$(date +%s)

    verify_symbols
    local verify_result=$?

    analyze_symbols
    export_symbols
    compare_symbols

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Create master report
    local master_report="$OUTPUT_DIR/verification-summary.txt"

    echo "=== ZipLock Android Symbol Verification Summary ===" > "$master_report"
    echo "Timestamp: $(date)" >> "$master_report"
    echo "Duration: ${duration}s" >> "$master_report"
    echo >> "$master_report"

    echo "Verification Steps:" >> "$master_report"
    echo "- Symbol verification: $([ $verify_result -eq 0 ] && echo "✓ PASSED" || echo "✗ FAILED")" >> "$master_report"
    echo "- Symbol analysis: $([ -f "$OUTPUT_DIR/symbol-analysis.txt" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$master_report"
    echo "- Symbol export: $([ -d "$OUTPUT_DIR/exports" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$master_report"
    echo "- Symbol comparison: $([ -f "$OUTPUT_DIR/symbol-comparison.txt" ] && echo "✓ Completed" || echo "✗ Failed")" >> "$master_report"
    echo >> "$master_report"

    echo "Output Files:" >> "$master_report"
    find "$OUTPUT_DIR" -name "*.txt" -type f | sort | while read -r file; do
        echo "- $(realpath --relative-to="$PROJECT_ROOT" "$file")" >> "$master_report"
    done

    if [ -d "$OUTPUT_DIR/exports" ]; then
        echo >> "$master_report"
        echo "Exported Files:" >> "$master_report"
        find "$OUTPUT_DIR/exports" -type f | sort | while read -r file; do
            echo "- $(realpath --relative-to="$PROJECT_ROOT" "$file")" >> "$master_report"
        done
    fi

    log_success "Complete verification finished in ${duration}s"
    log_info "Master summary: $master_report"

    return $verify_result
}

# Main function
main() {
    local command=""
    local verbose=false
    local strict=false

    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            -v|--verbose)
                verbose=true
                shift
                ;;
            -o|--output)
                OUTPUT_DIR="$2"
                shift 2
                ;;
            --strict)
                strict=true
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            *)
                if [ -z "$command" ]; then
                    command="$1"
                else
                    log_error "Unknown option: $1"
                    show_usage
                    exit 1
                fi
                shift
                ;;
        esac
    done

    if [ -z "$command" ]; then
        log_error "Command is required"
        show_usage
        exit 1
    fi

    # Set verbose output
    if [ "$verbose" = true ]; then
        set -x
    fi

    # Check dependencies and initialize
    check_dependencies
    init_environment

    # Run requested command
    local exit_code=0

    case "$command" in
        "verify")
            verify_symbols || exit_code=$?
            ;;
        "analyze")
            analyze_symbols
            ;;
        "export")
            export_symbols
            ;;
        "compare")
            compare_symbols
            ;;
        "all")
            run_all_verification || exit_code=$?
            ;;
        *)
            log_error "Invalid command: $command"
            show_usage
            exit 1
            ;;
    esac

    # Handle strict mode
    if [ "$strict" = true ] && [ $exit_code -ne 0 ]; then
        log_error "Verification failed in strict mode"
        exit $exit_code
    fi

    log_info "Results available in: $OUTPUT_DIR"
    exit $exit_code
}

# Run main function
main "$@"

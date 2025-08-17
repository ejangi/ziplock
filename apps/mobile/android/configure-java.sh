#!/bin/bash

# ZipLock Android Java Configuration Script
# Automatically detects and configures Java home for Android development

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

print_status() { echo -e "${BLUE}[INFO]${NC} $1"; }
print_success() { echo -e "${GREEN}[✓]${NC} $1"; }
print_warning() { echo -e "${YELLOW}[⚠]${NC} $1"; }
print_error() { echo -e "${RED}[✗]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
GRADLE_PROPERTIES="$SCRIPT_DIR/gradle.properties"

echo "================================================================"
echo "           ZipLock Android Java Configuration"
echo "================================================================"
echo ""

# Function to detect Java installations
detect_java_installations() {
    local java_homes=()

    # Check current JAVA_HOME
    if [[ -n "$JAVA_HOME" && -x "$JAVA_HOME/bin/java" ]]; then
        java_homes+=("$JAVA_HOME")
    fi

    # Check system Java
    if command -v java >/dev/null 2>&1; then
        local java_path=$(which java)
        local real_path=$(readlink -f "$java_path" 2>/dev/null || echo "$java_path")
        local java_home=$(dirname "$(dirname "$real_path")")
        if [[ -x "$java_home/bin/java" ]]; then
            java_homes+=("$java_home")
        fi
    fi

    # Common Java installation paths
    local common_paths=(
        "/usr/lib/jvm/java-21-openjdk"
        "/usr/lib/jvm/java-21-openjdk-amd64"
        "/usr/lib/jvm/java-17-openjdk"
        "/usr/lib/jvm/java-17-openjdk-amd64"
        "/usr/lib/jvm/java-11-openjdk"
        "/usr/lib/jvm/java-11-openjdk-amd64"
        "/usr/lib/jvm/default-java"
        "/Library/Java/JavaVirtualMachines/*/Contents/Home"
        "/System/Library/Java/JavaVirtualMachines/*/Contents/Home"
        "/opt/homebrew/opt/openjdk*/libexec/openjdk.jdk/Contents/Home"
        "/usr/local/opt/openjdk*/libexec/openjdk.jdk/Contents/Home"
        "C:/Program Files/Java/jdk-*"
        "C:/Program Files/Eclipse Adoptium/jdk-*"
        "C:/Program Files/OpenJDK/jdk-*"
    )

    for path in "${common_paths[@]}"; do
        # Handle wildcards
        for expanded_path in $path; do
            if [[ -d "$expanded_path" && -x "$expanded_path/bin/java" ]]; then
                java_homes+=("$expanded_path")
            fi
        done
    done

    # Remove duplicates and return unique paths
    printf '%s\n' "${java_homes[@]}" | sort -u
}

# Function to get Java version from a Java home
get_java_version() {
    local java_home="$1"
    if [[ -x "$java_home/bin/java" ]]; then
        "$java_home/bin/java" -version 2>&1 | head -1 | cut -d'"' -f2
    else
        echo "unknown"
    fi
}

# Function to check if Java version is compatible
is_compatible_version() {
    local version="$1"
    local major=$(echo "$version" | cut -d'.' -f1)

    # Check if it's a number and in valid range
    if [[ "$major" =~ ^[0-9]+$ ]] && [[ "$major" -ge 11 ]] && [[ "$major" -le 21 ]]; then
        return 0
    else
        return 1
    fi
}

# Main detection and configuration
main() {
    print_status "Detecting Java installations..."

    local java_installations
    java_installations=$(detect_java_installations)

    if [[ -z "$java_installations" ]]; then
        print_error "No Java installations found"
        echo ""
        print_status "Please install Java 11, 17, or 21:"
        echo "  • Ubuntu/Debian: sudo apt install openjdk-17-jdk"
        echo "  • macOS: brew install openjdk@17"
        echo "  • Windows: Download from https://adoptium.net/"
        echo "  • Or use Android Studio's embedded JDK"
        exit 1
    fi

    echo ""
    print_status "Found Java installations:"

    local compatible_javas=()
    local java_info=()
    local counter=1

    while IFS= read -r java_home; do
        local version=$(get_java_version "$java_home")
        local compat_status=""

        if is_compatible_version "$version"; then
            compat_status="${GREEN}✓ Compatible${NC}"
            compatible_javas+=("$java_home")
        else
            compat_status="${YELLOW}⚠ Not recommended${NC}"
        fi

        echo -e "  $counter. $java_home"
        echo -e "     Version: $version ($compat_status)"
        java_info+=("$java_home|$version")
        ((counter++))
    done <<< "$java_installations"

    echo ""

    # If no compatible Java found
    if [[ ${#compatible_javas[@]} -eq 0 ]]; then
        print_error "No compatible Java versions found (need Java 11-21)"
        print_status "Consider installing a compatible Java version or using Android Studio's embedded JDK"
        exit 1
    fi

    # Auto-select best Java version
    local selected_java=""
    local best_version=""

    # Prefer Java 17 > 11 > 21 > others
    for java_home in "${compatible_javas[@]}"; do
        local version=$(get_java_version "$java_home")
        local major=$(echo "$version" | cut -d'.' -f1)

        if [[ "$major" == "17" ]]; then
            selected_java="$java_home"
            best_version="$version"
            break
        elif [[ "$major" == "11" && -z "$selected_java" ]]; then
            selected_java="$java_home"
            best_version="$version"
        elif [[ "$major" == "21" && -z "$selected_java" ]]; then
            selected_java="$java_home"
            best_version="$version"
        elif [[ -z "$selected_java" ]]; then
            selected_java="$java_home"
            best_version="$version"
        fi
    done

    print_success "Selected Java: $selected_java (Version: $best_version)"

    # Configure Gradle properties
    print_status "Configuring Gradle properties..."

    # Check if org.gradle.java.home is already set correctly
    if grep -q "^org.gradle.java.home=" "$GRADLE_PROPERTIES"; then
        # Update existing setting
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed
            sed -i '' "s|^org.gradle.java.home=.*|org.gradle.java.home=$selected_java|" "$GRADLE_PROPERTIES"
        else
            # Linux sed
            sed -i "s|^org.gradle.java.home=.*|org.gradle.java.home=$selected_java|" "$GRADLE_PROPERTIES"
        fi
        print_success "Updated existing Java home configuration"
    else
        # Add new setting
        echo "" >> "$GRADLE_PROPERTIES"
        echo "# Configured Java home for Android development" >> "$GRADLE_PROPERTIES"
        echo "org.gradle.java.home=$selected_java" >> "$GRADLE_PROPERTIES"
        print_success "Added Java home configuration"
    fi

    # Set environment variables for current session
    export JAVA_HOME="$selected_java"
    export PATH="$JAVA_HOME/bin:$PATH"

    print_success "Environment configured for current session"

    echo ""
    print_status "Testing configuration..."

    # Test Java
    if "$selected_java/bin/java" -version >/dev/null 2>&1; then
        print_success "Java is working correctly"
    else
        print_error "Java test failed"
        exit 1
    fi

    # Test Gradle wrapper if available
    if [[ -x "$SCRIPT_DIR/gradlew" ]]; then
        print_status "Testing Gradle wrapper..."
        if cd "$SCRIPT_DIR" && ./gradlew --version >/dev/null 2>&1; then
            print_success "Gradle wrapper is working correctly"
        else
            print_warning "Gradle wrapper test failed (may need Android Studio sync)"
        fi
    fi

    echo ""
    print_success "Configuration complete!"
    echo ""
    print_status "Next steps:"
    echo "1. Open Android Studio"
    echo "2. File → Sync Project with Gradle Files"
    echo "3. Create an emulator and run the app"
    echo ""
    print_status "For persistent environment variables, add to your shell profile:"
    echo "   export JAVA_HOME=\"$selected_java\""
    echo "   export PATH=\"\$JAVA_HOME/bin:\$PATH\""
}

# Help function
show_help() {
    echo "ZipLock Android Java Configuration Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h     Show this help message"
    echo "  --list, -l     List detected Java installations"
    echo "  --reset, -r    Reset Java configuration"
    echo ""
    echo "This script automatically detects and configures the best Java"
    echo "installation for Android development with ZipLock."
}

# List Java installations
list_javas() {
    print_status "Detecting Java installations..."
    local java_installations
    java_installations=$(detect_java_installations)

    if [[ -z "$java_installations" ]]; then
        print_error "No Java installations found"
        exit 1
    fi

    echo ""
    while IFS= read -r java_home; do
        local version=$(get_java_version "$java_home")
        local compat_status=""

        if is_compatible_version "$version"; then
            compat_status="${GREEN}✓ Compatible${NC}"
        else
            compat_status="${YELLOW}⚠ Not recommended${NC}"
        fi

        echo -e "$java_home"
        echo -e "  Version: $version ($compat_status)"
        echo ""
    done <<< "$java_installations"
}

# Reset Java configuration
reset_config() {
    print_status "Resetting Java configuration..."

    if grep -q "^org.gradle.java.home=" "$GRADLE_PROPERTIES"; then
        if [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS sed
            sed -i '' '/^org.gradle.java.home=/d' "$GRADLE_PROPERTIES"
        else
            # Linux sed
            sed -i '/^org.gradle.java.home=/d' "$GRADLE_PROPERTIES"
        fi
        print_success "Removed Java home configuration from gradle.properties"
    else
        print_warning "No Java home configuration found in gradle.properties"
    fi

    echo ""
    print_status "Java configuration reset. Run without arguments to reconfigure."
}

# Parse command line arguments
case "${1:-}" in
    --help|-h)
        show_help
        exit 0
        ;;
    --list|-l)
        list_javas
        exit 0
        ;;
    --reset|-r)
        reset_config
        exit 0
        ;;
    "")
        main
        ;;
    *)
        print_error "Unknown option: $1"
        echo ""
        show_help
        exit 1
        ;;
esac

#!/bin/bash

# ZipLock Android Project Setup Verification Script
# This script checks if your development environment is ready for Android development

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
PROJECT_ROOT="$(dirname "$(dirname "$SCRIPT_DIR")")"
ANDROID_DIR="$PROJECT_ROOT/apps/mobile/android"

echo "================================================================"
echo "           ZipLock Android Development Setup Check"
echo "================================================================"
echo ""

# Check 1: Java Installation
print_status "Checking Java installation..."
if command -v java >/dev/null 2>&1; then
    java_version=$(java -version 2>&1 | head -1 | cut -d'"' -f2)
    print_success "Java found: $java_version"

    # Check Java version compatibility
    java_major=$(echo "$java_version" | cut -d'.' -f1)
    if [[ "$java_major" -ge 11 && "$java_major" -le 21 ]]; then
        print_success "Java version is compatible with Android development"
    else
        print_warning "Java version may be incompatible (recommended: 11-21)"
    fi

    if [[ -n "$JAVA_HOME" ]]; then
        print_success "JAVA_HOME is set: $JAVA_HOME"
    else
        print_warning "JAVA_HOME is not set (Android Studio will use its embedded JDK)"
    fi
else
    print_error "Java not found in PATH"
    print_warning "Install OpenJDK 11, 17, or 21, or use Android Studio's embedded JDK"
fi
echo ""

# Check 2: Android SDK
print_status "Checking Android SDK..."
if [[ -n "$ANDROID_HOME" ]]; then
    print_success "ANDROID_HOME is set: $ANDROID_HOME"

    if [[ -d "$ANDROID_HOME/platform-tools" ]]; then
        print_success "Platform tools found"
    else
        print_warning "Platform tools not found in ANDROID_HOME"
    fi

    if [[ -d "$ANDROID_HOME/emulator" ]]; then
        print_success "Emulator found"
    else
        print_warning "Emulator not found in ANDROID_HOME"
    fi
else
    print_warning "ANDROID_HOME not set (will be configured by Android Studio)"
fi
echo ""

# Check 3: ADB
print_status "Checking ADB (Android Debug Bridge)..."
if command -v adb >/dev/null 2>&1; then
    adb_version=$(adb version | head -1)
    print_success "ADB found: $adb_version"
else
    print_warning "ADB not found in PATH (will be available after SDK installation)"
fi
echo ""

# Check 4: Project Structure
print_status "Checking project structure..."

# Check for essential files
files_to_check=(
    "build.gradle"
    "settings.gradle"
    "gradle.properties"
    "app/build.gradle"
    "app/src/main/AndroidManifest.xml"
    "app/src/main/java/com/ziplock/SplashActivity.kt"
    "app/src/main/java/com/ziplock/MainActivity.kt"
    "app/src/main/res/values/strings.xml"
    "app/src/main/res/values/colors.xml"
    "app/src/main/res/values/themes.xml"
)

all_files_present=true
for file in "${files_to_check[@]}"; do
    if [[ -f "$SCRIPT_DIR/$file" ]]; then
        print_success "Found: $file"
    else
        print_error "Missing: $file"
        all_files_present=false
    fi
done

if $all_files_present; then
    print_success "All essential project files are present"
else
    print_error "Some project files are missing"
fi
echo ""

# Check 5: Native Libraries
print_status "Checking native libraries..."
native_libs_dir="$ANDROID_DIR/app/src/main/jniLibs"

if [[ -d "$native_libs_dir" ]]; then
    print_success "Native libs directory exists"

    architectures=("arm64-v8a" "armeabi-v7a" "x86_64" "x86")
    libs_found=0

    for arch in "${architectures[@]}"; do
        lib_path="$native_libs_dir/$arch/libziplock_shared.so"
        if [[ -f "$lib_path" ]]; then
            size=$(du -h "$lib_path" | cut -f1)
            print_success "Found $arch library: $size"
            ((libs_found++))
        else
            print_warning "Missing $arch library"
        fi
    done

    if [[ $libs_found -gt 0 ]]; then
        print_success "Found $libs_found native libraries"
    else
        print_error "No native libraries found"
        print_warning "Run: ./scripts/build/build-android-docker.sh build"
    fi

    # Check header file
    if [[ -f "$native_libs_dir/ziplock.h" ]]; then
        print_success "Found C header file"
    else
        print_warning "Missing C header file"
    fi
else
    print_error "Native libs directory not found"
    print_warning "Run: ./scripts/build/build-android-docker.sh build"
fi
echo ""

# Check 6: Gradle Wrapper
print_status "Checking Gradle wrapper..."
if [[ -f "$ANDROID_DIR/gradlew" ]]; then
    print_success "Gradle wrapper script found"

    if [[ -x "$ANDROID_DIR/gradlew" ]]; then
        print_success "Gradle wrapper is executable"
    else
        print_warning "Gradle wrapper is not executable"
        print_warning "Run: chmod +x gradlew"
    fi
else
    print_error "Gradle wrapper script not found"
fi

if [[ -f "$ANDROID_DIR/gradle/wrapper/gradle-wrapper.jar" ]]; then
    jar_size=$(du -h "$ANDROID_DIR/gradle/wrapper/gradle-wrapper.jar" | cut -f1)
    print_success "Gradle wrapper JAR found: $jar_size"
else
    print_error "Gradle wrapper JAR not found"
fi

if [[ -f "$ANDROID_DIR/gradle/wrapper/gradle-wrapper.properties" ]]; then
    gradle_version=$(grep "distributionUrl" "$ANDROID_DIR/gradle/wrapper/gradle-wrapper.properties" | sed 's/.*gradle-\([0-9.]*\).*/\1/')
    print_success "Gradle version configured: $gradle_version"

    # Check Gradle compatibility with Java
    if command -v java >/dev/null 2>&1; then
        java_major=$(java -version 2>&1 | head -1 | cut -d'"' -f2 | cut -d'.' -f1)
        if [[ "$gradle_version" == "8.5" && "$java_major" -ge 11 && "$java_major" -le 21 ]]; then
            print_success "Gradle and Java versions are compatible"
        elif [[ "$gradle_version" == "8.2" && "$java_major" -le 19 ]]; then
            print_success "Gradle and Java versions are compatible"
        else
            print_warning "Gradle $gradle_version and Java $java_major may be incompatible"
            print_warning "See JAVA_COMPATIBILITY.md for solutions"
        fi
    fi
else
    print_error "Gradle wrapper properties not found"
fi
echo ""

# Check 7: System Resources
print_status "Checking system resources..."

# RAM
if command -v free >/dev/null 2>&1; then
    total_ram=$(free -h | awk '/^Mem:/ {print $2}')
    print_success "Total RAM: $total_ram"

    # Parse RAM and check if it's sufficient
    ram_gb=$(free -g | awk '/^Mem:/ {print $2}')
    if [[ $ram_gb -ge 8 ]]; then
        print_success "RAM is sufficient for Android development"
    else
        print_warning "RAM may be insufficient for optimal performance (8GB+ recommended)"
    fi
elif command -v sysctl >/dev/null 2>&1; then
    # macOS
    total_ram=$(sysctl -n hw.memsize | awk '{print $1/1024/1024/1024 " GB"}')
    print_success "Total RAM: $total_ram GB"
else
    print_warning "Could not determine RAM size"
fi

# Disk space
disk_space=$(df -h "$SCRIPT_DIR" | awk 'NR==2 {print $4}')
print_success "Available disk space: $disk_space"
echo ""

# Check 8: Virtualization Support (for emulator)
print_status "Checking virtualization support..."
if [[ -f /proc/cpuinfo ]]; then
    if grep -q "vmx\|svm" /proc/cpuinfo; then
        print_success "Hardware virtualization supported"
    else
        print_warning "Hardware virtualization not detected"
        print_warning "Emulator performance may be poor"
    fi
elif command -v sysctl >/dev/null 2>&1; then
    # macOS
    if sysctl -n machdep.cpu.features | grep -q VMX; then
        print_success "Hardware virtualization supported"
    else
        print_warning "Hardware virtualization not detected"
    fi
else
    print_warning "Could not check virtualization support"
fi
echo ""

# Summary
echo "================================================================"
print_status "Setup Summary"
echo "================================================================"

if $all_files_present && [[ -d "$native_libs_dir" ]] && [[ -f "$ANDROID_DIR/gradlew" ]]; then
    print_success "✅ Project structure is ready"
else
    print_error "❌ Project structure needs attention"
fi

if [[ $libs_found -gt 0 ]]; then
    print_success "✅ Native libraries are available"
else
    print_error "❌ Native libraries need to be built"
fi

echo ""
print_status "Next Steps:"
echo "1. Install Android Studio from: https://developer.android.com/studio"
echo "2. Open this project in Android Studio"
echo "3. Install required SDK components"
echo "4. Create an Android Virtual Device (AVD)"
echo "5. Run the app on the emulator"
echo ""

if [[ $libs_found -eq 0 ]]; then
    print_warning "Build native libraries first:"
    echo "   cd $PROJECT_ROOT"
    echo "   ./scripts/build/build-android-docker.sh build"
    echo ""
fi

# Java compatibility check
if command -v java >/dev/null 2>&1; then
    java_major=$(java -version 2>&1 | head -1 | cut -d'"' -f2 | cut -d'.' -f1)
    if [[ "$java_major" -gt 21 || "$java_major" -lt 11 ]]; then
        print_warning "For Java compatibility issues, see: JAVA_COMPATIBILITY.md"
        echo ""
    fi
fi

print_status "For detailed setup instructions, see: SETUP.md"
echo ""

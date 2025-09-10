FROM ubuntu:22.04

# Prevent interactive prompts during package installation
ENV DEBIAN_FRONTEND=noninteractive
ENV TZ=UTC

# Install system dependencies in a single layer
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    curl \
    ca-certificates \
    file \
    binutils \
    unzip \
    openjdk-17-jdk \
    python3 \
    python3-pip \
    git \
    wget \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

# Install Rust toolchain with Android targets
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    --default-toolchain stable
ENV PATH="/root/.cargo/bin:${PATH}"

# Add Android targets
RUN rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android i686-linux-android

# Install Android NDK
ENV ANDROID_NDK_VERSION=25.2.9519653
ENV ANDROID_NDK_HOME=/opt/android-ndk
RUN curl -L https://dl.google.com/android/repository/android-ndk-r25c-linux.zip \
    -o android-ndk.zip \
    && unzip android-ndk.zip \
    && mv android-ndk-r25c ${ANDROID_NDK_HOME} \
    && rm android-ndk.zip

# Add NDK to PATH
ENV PATH="${ANDROID_NDK_HOME}/toolchains/llvm/prebuilt/linux-x86_64/bin:${PATH}"

# Set NDK environment variables
ENV NDK_ROOT=${ANDROID_NDK_HOME}
ENV ANDROID_API_LEVEL=21

# Configure Cargo for Android cross-compilation
RUN mkdir -p /root/.cargo && \
    echo '[target.aarch64-linux-android]' > /root/.cargo/config.toml && \
    echo 'ar = "llvm-ar"' >> /root/.cargo/config.toml && \
    echo 'linker = "aarch64-linux-android21-clang"' >> /root/.cargo/config.toml && \
    echo 'rustflags = [' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=--target=aarch64-linux-android21",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-llog",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lm",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-ldl",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lc"' >> /root/.cargo/config.toml && \
    echo ']' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml && \
    echo '[target.armv7-linux-androideabi]' >> /root/.cargo/config.toml && \
    echo 'ar = "llvm-ar"' >> /root/.cargo/config.toml && \
    echo 'linker = "armv7a-linux-androideabi21-clang"' >> /root/.cargo/config.toml && \
    echo 'rustflags = [' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=--target=armv7a-linux-androideabi21",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-llog",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lm",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-ldl",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lc"' >> /root/.cargo/config.toml && \
    echo ']' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml && \
    echo '[target.x86_64-linux-android]' >> /root/.cargo/config.toml && \
    echo 'ar = "llvm-ar"' >> /root/.cargo/config.toml && \
    echo 'linker = "x86_64-linux-android21-clang"' >> /root/.cargo/config.toml && \
    echo 'rustflags = [' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=--target=x86_64-linux-android21",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-llog",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lm",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-ldl",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lc"' >> /root/.cargo/config.toml && \
    echo ']' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml && \
    echo '[target.i686-linux-android]' >> /root/.cargo/config.toml && \
    echo 'ar = "llvm-ar"' >> /root/.cargo/config.toml && \
    echo 'linker = "i686-linux-android21-clang"' >> /root/.cargo/config.toml && \
    echo 'rustflags = [' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=--target=i686-linux-android21",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-llog",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lm",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-ldl",' >> /root/.cargo/config.toml && \
    echo '    "-C", "link-arg=-lc"' >> /root/.cargo/config.toml && \
    echo ']' >> /root/.cargo/config.toml

# Set additional environment variables for C compilation
ENV CC_aarch64_linux_android="aarch64-linux-android21-clang"
ENV CXX_aarch64_linux_android="aarch64-linux-android21-clang++"
ENV AR_aarch64_linux_android="llvm-ar"

ENV CC_armv7_linux_androideabi="armv7a-linux-androideabi21-clang"
ENV CXX_armv7_linux_androideabi="armv7a-linux-androideabi21-clang++"
ENV AR_armv7_linux_androideabi="llvm-ar"

ENV CC_x86_64_linux_android="x86_64-linux-android21-clang"
ENV CXX_x86_64_linux_android="x86_64-linux-android21-clang++"
ENV AR_x86_64_linux_android="llvm-ar"

ENV CC_i686_linux_android="i686-linux-android21-clang"
ENV CXX_i686_linux_android="i686-linux-android21-clang++"
ENV AR_i686_linux_android="llvm-ar"

# Set Java 17 as the default Java version
ENV JAVA_HOME="/usr/lib/jvm/java-17-openjdk-amd64"
ENV PATH="${JAVA_HOME}/bin:${PATH}"

# Set cargo target directory
ENV CARGO_TARGET_DIR="/workspace/target"

# Verify installation
RUN rustc --version && \
    cargo --version && \
    aarch64-linux-android21-clang --version && \
    java -version && \
    echo "JAVA_HOME: $JAVA_HOME"

# Create workspace directory
WORKDIR /workspace

# Set default command
CMD ["/bin/bash"]

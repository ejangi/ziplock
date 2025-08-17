# TODO

Run one of the following prompts in Zed's agent panel and when you're ready to bump the version, use `./scripts/version/update-version.sh patch "<My feature update here>"`:


## Linux
- Taking into consideration `docs/technical.md` and `docs/architecture.md`, Can you please `cargo check` the linux app, look at each of the warnings and work out how to correct them? We have a shared library, so any functionality where it could be shared between platforms we should move into that location. We also need to evaluate whether each item in the warnings should still exist and is just not being used correctly. If the functionality really is redundant then please remove it.
- Taking into consideration `docs/technical.md`, `docs/architecture.md` and `docs/technical/build.md`, can you please fix the github workflow, which is currently failing on job "Create Debian Package" at the "Create Debian package" step with the following logs:
```
echo "Creating Debian package from installation structure..."

# Validate prerequisites
if [ ! -f "target/install/usr/bin/ziplock" ]; then
  echo "ERROR: Required binary not found in installation structure"
  exit 1
fi

if [ ! -f "target/install/usr/lib/libziplock_shared.so" ]; then
  echo "ERROR: Required shared library not found in installation structure"
  exit 1
fi

# Get current user and group IDs for proper permission handling
USER_ID=$(id -u)
GROUP_ID=$(id -g)

# Run with proper error handling and permission management
set -e
docker run --rm \
  -v $PWD:/workspace \
  -e USER_ID="$USER_ID" \
  -e GROUP_ID="$GROUP_ID" \
  ghcr.io/ejangi/ziplock/ubuntu-builder:latest \
  bash -c "
    set -euo pipefail

    # Copy workspace to avoid permission issues with git
    echo 'Copying workspace to avoid git permission conflicts...'
    cp -r /workspace /tmp/build
    cd /tmp/build

    # Validate copied structure
    if [ ! -f 'target/install/usr/bin/ziplock' ]; then
      echo 'ERROR: Binary not found in copied workspace'
      exit 1
    fi

    # Run packaging
    chmod +x scripts/build/package-deb.sh
    if ! ./scripts/build/package-deb.sh --arch amd64; then
      echo 'ERROR: Debian packaging script failed'
      exit 1
    fi

    # Verify package was created
    if [ ! -f target/ziplock_*_amd64.deb ]; then
      echo 'ERROR: Debian package was not created'
      ls -la target/ || true
      exit 1
    fi

    # Copy results back to workspace with correct ownership
    echo 'Copying build results back...'
    cp target/ziplock_*_amd64.deb /workspace/target/

    # Restore original ownership
    chown -R \$USER_ID:\$GROUP_ID /workspace

    echo 'Debian package creation completed successfully'
  "

# Final verification of build results
echo "Verifying Debian package build results..."
if ! ls target/ziplock_*_amd64.deb 1> /dev/null 2>&1; then
  echo "ERROR: Debian package .deb not found after build"
  echo "Contents of target directory:"
  ls -la target/ || true
  exit 1
fi

# Verify package integrity
if ! dpkg-deb --info target/ziplock_*_amd64.deb > /dev/null; then
  echo "ERROR: Debian package is corrupted or invalid"
  exit 1
fi

echo "Debian package build completed successfully"
shell: /usr/bin/bash -e {0}
env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1
Creating Debian package from installation structure...
Unable to find image 'ghcr.io/ejangi/ziplock/ubuntu-builder:latest' locally
latest: Pulling from ejangi/ziplock/ubuntu-builder
a3be5d4ce401: Pulling fs layer
9ff313c28ee4: Pulling fs layer
bbe91e4faca3: Pulling fs layer
4f4fb700ef54: Pulling fs layer
b415a83a3d68: Pulling fs layer
3fcb1deadfa7: Pulling fs layer
4f4fb700ef54: Waiting
b415a83a3d68: Waiting
3fcb1deadfa7: Waiting
a3be5d4ce401: Download complete
4f4fb700ef54: Verifying Checksum
4f4fb700ef54: Download complete
b415a83a3d68: Verifying Checksum
b415a83a3d68: Download complete
3fcb1deadfa7: Verifying Checksum
3fcb1deadfa7: Download complete
a3be5d4ce401: Pull complete
9ff313c28ee4: Verifying Checksum
9ff313c28ee4: Download complete
bbe91e4faca3: Verifying Checksum
bbe91e4faca3: Download complete
9ff313c28ee4: Pull complete
bbe91e4faca3: Pull complete
4f4fb700ef54: Pull complete
b415a83a3d68: Pull complete
3fcb1deadfa7: Pull complete
Digest: sha256:7efdbabc7705b66810cc797db9cb86bcf1dcc5d6d90b95b2f88b0eaa2dca23c1
Status: Downloaded newer image for ghcr.io/ejangi/ziplock/ubuntu-builder:latest
Copying workspace to avoid git permission conflicts...
[INFO] Starting Debian package creation...
[INFO] Package: ziplock v0.2.8 (amd64)
[INFO] Checking packaging dependencies...
[SUCCESS] Packaging dependencies satisfied
[INFO] Verifying build artifacts...
[ERROR] Required file missing: /tmp/build/target/install/usr/share/mime/packages/ziplock.xml
ERROR: Debian packaging script failed
Error: Process completed with exit code 1.
```


## Android
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` can you please update the Android "Open Archive" screen so that when the user enters a passphrase and hits the "Enter" key on the keyboard that it performs the same action as clicking on the "Open Archive" button on screen.
- Taking into consideration `docs/design.md`, `docs/technical.md` and `docs/technical/android.md` can you please ensure that the Android app closes the archive correctly when the user exits the app. Is there the idea of a quick background process to finish that sort of them on Android?

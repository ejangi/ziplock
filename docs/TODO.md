# TODO

- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, can we ensure that for secure notes we are using a textarea/editor, not just a simple one line text input?
- The github workflow `Test Suite (beta)` is failing. When it runs the `cargo clippy --all-targets --all-features -- -D warnings` command it is outputting the error info:
```
warning: atk-sys@0.18.2:
error: failed to run custom build command for `atk-sys v0.18.2`
note: To improve backtraces for build dependencies, set the CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true environment variable to enable debug information generation.

Caused by:
  process didn't exit successfully: `/home/runner/work/ziplock/ziplock/target/debug/build/atk-sys-5b1b1079bc6976b0/build-script-build` (exit status: 1)
  --- stdout
  cargo:rerun-if-env-changed=ATK_NO_PKG_CONFIG
  cargo:rerun-if-env-changed=PKG_CONFIG_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG
  cargo:rerun-if-env-changed=PKG_CONFIG
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_PATH
  cargo:rerun-if-env-changed=PKG_CONFIG_PATH
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_LIBDIR
  cargo:rerun-if-env-changed=PKG_CONFIG_LIBDIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR_x86_64-unknown-linux-gnu
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR_x86_64_unknown_linux_gnu
  cargo:rerun-if-env-changed=HOST_PKG_CONFIG_SYSROOT_DIR
  cargo:rerun-if-env-changed=PKG_CONFIG_SYSROOT_DIR
  cargo:warning=
  pkg-config exited with status code 1
  > PKG_CONFIG_ALLOW_SYSTEM_CFLAGS=1 pkg-config --libs --cflags atk 'atk >= 2.28'

  The system library `atk` required by crate `atk-sys` was not found.
  The file `atk.pc` needs to be installed and the PKG_CONFIG_PATH environment variable must contain its parent directory.
  The PKG_CONFIG_PATH environment variable is not set.

  HINT: if you have installed the library, try setting PKG_CONFIG_PATH to the directory containing `atk.pc`.

warning: build failed, waiting for other jobs to finish...
Error: Process completed with exit code 101.
```
- Taking into consideration the `ziplock/README.md` and `ziplock/docs/*.md` files, should we look at making the IpcClient a shared component with C-bindings for use in other languages?

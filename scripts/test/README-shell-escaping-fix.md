# Shell Escaping Fix for Arch Package Build

## Problem

The GitHub Actions workflow was failing with a shell syntax error during the "Create Arch package structure" step:

```
/home/runner/work/_temp/xxx.sh: line 79: syntax error near unexpected token `('
Error: Process completed with exit code 2.
```

This was caused by complex shell escaping issues when trying to update the `sha256sums` line in the PKGBUILD file using a `sed` command within nested Docker bash commands.

## Root Cause

The problematic command was:
```bash
sed "s/^sha256sums=.*/sha256sums=('$SHA256')/" PKGBUILD > PKGBUILD.tmp
```

When this command was executed within multiple layers of shell escaping (GitHub Actions → Docker → bash -c → sudo -u builder bash -c), the parentheses and quotes caused syntax errors.

## Solution

Replaced the complex `sed` command with a simpler `printf`-based approach that avoids shell escaping issues:

### Before (Problematic)
```bash
# Complex sed with escaping issues
sed "s/^sha256sums=.*/sha256sums=('$SHA256')/" PKGBUILD > PKGBUILD.tmp
mv PKGBUILD.tmp PKGBUILD
```

### After (Working Solution)
```bash
# Printf approach - no escaping issues
cp PKGBUILD PKGBUILD.backup
grep -v "^sha256sums=" PKGBUILD.backup > PKGBUILD.tmp
printf "sha256sums=('%s')\n" "$SHA256" >> PKGBUILD.tmp
mv PKGBUILD.tmp PKGBUILD
rm PKGBUILD.backup
```

## How the Fix Works

1. **Backup the original PKGBUILD**: `cp PKGBUILD PKGBUILD.backup`
2. **Copy all lines except sha256sums**: `grep -v "^sha256sums=" PKGBUILD.backup > PKGBUILD.tmp`
3. **Add new sha256sums line**: `printf "sha256sums=('%s')\n" "$SHA256" >> PKGBUILD.tmp`
4. **Replace original file**: `mv PKGBUILD.tmp PKGBUILD`
5. **Cleanup**: `rm PKGBUILD.backup`

## Benefits

- ✅ **No shell escaping issues**: `printf` handles quoting automatically
- ✅ **More readable**: Clear step-by-step process
- ✅ **Safer**: Preserves file structure and handles edge cases
- ✅ **Reliable**: Works consistently across different shell environments
- ✅ **Maintainable**: Easy to understand and modify

## Testing

Run the test scripts to verify the fix:

```bash
# Test the printf approach locally
./scripts/test/test-printf-approach.sh

# Test the specific shell escaping scenarios
./scripts/test/test-shell-escaping.sh

# Test the complete workflow locally
./scripts/test/test-arch-packaging-local.sh
```

## Edge Cases Handled

- Multi-line `sha256sums` arrays
- Special characters in SHA256 hashes
- Different shell environments (bash, sh, etc.)
- Nested command execution in Docker containers

## File Changes

The fix was applied in `.github/workflows/unified-release.yml` in the "Create Arch package" step, specifically in the Docker command that updates the PKGBUILD file.

## Alternative Approaches Considered

1. **Heredoc**: Would work but requires more complex structure
2. **AWK**: More powerful but overkill for this simple task
3. **Python/Perl**: Would require additional dependencies
4. **File templating**: More complex for this use case

The `printf` approach was chosen for its simplicity, reliability, and zero additional dependencies.
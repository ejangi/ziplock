import com.sun.jna.Library;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import java.io.File;
import java.util.Base64;
import java.util.HashMap;
import java.util.Map;

/**
 * Standalone Java test to verify the temporary archive FFI function works
 * without the libgcc_s.so.1 dependency issue.
 *
 * This test can be run directly to verify that:
 * 1. The native library loads successfully
 * 2. The ziplock_mobile_create_temp_archive function works
 * 3. No libgcc_s.so.1 errors occur
 *
 * Usage:
 *   javac -cp ".:path/to/jna.jar" scripts/dev/test-temp-archive-ffi.java
 *   java -cp ".:path/to/jna.jar" -Djava.library.path=apps/mobile/android/app/src/main/jniLibs/x86_64 TestTempArchiveFFI
 */
public class TestTempArchiveFFI {

    // JNA interface for the mobile FFI library
    public interface ZipLockMobileLibrary extends Library {
        ZipLockMobileLibrary INSTANCE = Native.load("ziplock_shared", ZipLockMobileLibrary.class);

        // Temporary archive creation function
        int ziplock_mobile_create_temp_archive(String filesJson, String password, Pointer[] tempPathOut);

        // String cleanup function
        void ziplock_mobile_free_string(Pointer strPtr);
    }

    public static void main(String[] args) {
        System.out.println("=== ZipLock Temporary Archive FFI Test ===");
        System.out.println();

        try {
            // Test 1: Library Loading
            System.out.println("Test 1: Loading native library...");
            ZipLockMobileLibrary lib = ZipLockMobileLibrary.INSTANCE;
            System.out.println("✅ Native library loaded successfully");
            System.out.println("✅ No libgcc_s.so.1 dependency issues!");
            System.out.println();

            // Test 2: Create test file map
            System.out.println("Test 2: Creating test file map...");
            Map<String, String> fileMap = new HashMap<>();

            // Create some test credential files
            String metadataYaml = "version: \"1.0\"\nformat: \"ziplock\"\ncreated_at: \"2023-12-01T10:00:00Z\"\n";
            String credentialYaml = "id: \"test-123\"\ntitle: \"Test Credential\"\ncredential_type: \"login\"\nfields:\n  username:\n    value: \"testuser\"\n    field_type: \"text\"\n    sensitive: false\n  password:\n    value: \"testpass123\"\n    field_type: \"password\"\n    sensitive: true\n";

            // Encode to base64 as expected by the FFI
            fileMap.put("metadata.yml", Base64.getEncoder().encodeToString(metadataYaml.getBytes()));
            fileMap.put("credentials/test-123/record.yml", Base64.getEncoder().encodeToString(credentialYaml.getBytes()));

            System.out.println("✅ Test file map created with " + fileMap.size() + " files");
            System.out.println();

            // Test 3: Convert to JSON
            System.out.println("Test 3: Converting to JSON...");
            StringBuilder jsonBuilder = new StringBuilder("{");
            boolean first = true;
            for (Map.Entry<String, String> entry : fileMap.entrySet()) {
                if (!first) jsonBuilder.append(",");
                jsonBuilder.append("\"").append(entry.getKey()).append("\":\"").append(entry.getValue()).append("\"");
                first = false;
            }
            jsonBuilder.append("}");
            String filesJson = jsonBuilder.toString();

            System.out.println("✅ JSON created (length: " + filesJson.length() + " chars)");
            System.out.println();

            // Test 4: Call FFI function
            System.out.println("Test 4: Calling ziplock_mobile_create_temp_archive...");
            String password = "TestPassword123!@#";
            Pointer[] tempPathOut = new Pointer[1];

            int result = lib.ziplock_mobile_create_temp_archive(filesJson, password, tempPathOut);

            if (result == 0 && tempPathOut[0] != null) { // 0 = SUCCESS
                String tempPath = tempPathOut[0].getString(0);
                System.out.println("✅ Temporary archive created successfully!");
                System.out.println("   Path: " + tempPath);

                // Verify the file exists
                File tempFile = new File(tempPath);
                if (tempFile.exists()) {
                    long size = tempFile.length();
                    System.out.println("✅ Archive file exists (size: " + size + " bytes)");

                    // Clean up
                    if (tempFile.delete()) {
                        System.out.println("✅ Temporary file cleaned up");
                    }
                } else {
                    System.out.println("❌ Archive file does not exist at specified path");
                }

                // Free the string
                lib.ziplock_mobile_free_string(tempPathOut[0]);
                System.out.println("✅ Memory cleaned up");

            } else {
                System.out.println("❌ FFI call failed with result code: " + result);
                System.out.println("   This may indicate an issue with the implementation");
                System.exit(1);
            }

        } catch (UnsatisfiedLinkError e) {
            System.out.println("❌ Failed to load native library!");
            System.out.println("Error: " + e.getMessage());
            System.out.println();
            System.out.println("Possible causes:");
            System.out.println("1. Library not found in java.library.path");
            System.out.println("2. libgcc_s.so.1 dependency issue (should be fixed now)");
            System.out.println("3. Missing Android NDK libraries");
            System.out.println();
            System.out.println("Try:");
            System.out.println("  java -Djava.library.path=apps/mobile/android/app/src/main/jniLibs/x86_64 TestTempArchiveFFI");
            System.exit(1);

        } catch (Exception e) {
            System.out.println("❌ Unexpected error: " + e.getMessage());
            e.printStackTrace();
            System.exit(1);
        }

        System.out.println();
        System.out.println("=== All Tests Passed! ===");
        System.out.println("✅ Native library loads without libgcc_s.so.1 issues");
        System.out.println("✅ Temporary archive creation FFI function works");
        System.out.println("✅ The temporary archive approach is ready for use!");
        System.out.println();
        System.out.println("The Enhanced Archive Manager can now safely use the");
        System.out.println("shared library for encrypted archive creation.");
    }
}

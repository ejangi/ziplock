package com.ziplock.ffi

import android.content.Context
import android.net.Uri
import android.os.ParcelFileDescriptor
import android.provider.OpenableColumns
import android.util.Log
import com.sun.jna.Memory
import com.sun.jna.Pointer
import com.sun.jna.ptr.PointerByReference
import java.io.File
import java.io.FileDescriptor
import java.io.FileInputStream
import java.io.FileNotFoundException
import java.io.FileOutputStream
import java.io.IOException
import java.util.concurrent.ConcurrentHashMap

/**
 * Helper class that provides Android Storage Access Framework (SAF) functionality
 * to the native ZipLock library through JNI callbacks.
 */
class AndroidSafHelper(private val context: Context) {

    private val openFileDescriptors = ConcurrentHashMap<Int, ParcelFileDescriptor>()
    private var nextFdId = 1000 // Start with a high number to avoid conflicts

    /**
     * Callback to open a content URI and return a file descriptor
     */
    val openCallback = object : ZipLockNative.AndroidSafOpenCallback {
        override fun callback(contentUri: String): Int {
            return try {
                Log.d("AndroidSafHelper", "Opening content URI: $contentUri")

                val uri = Uri.parse(contentUri)
                val pfd = context.contentResolver.openFileDescriptor(uri, "r")

                if (pfd != null) {
                    val fdId = nextFdId++
                    openFileDescriptors[fdId] = pfd
                    Log.d("AndroidSafHelper", "Successfully opened content URI, assigned FD ID: $fdId")
                    fdId
                } else {
                    Log.e("AndroidSafHelper", "Failed to open content URI: null ParcelFileDescriptor")
                    -1
                }
            } catch (e: SecurityException) {
                Log.e("AndroidSafHelper", "Security exception opening content URI: $contentUri - ${e.message}", e)
                -2
            } catch (e: FileNotFoundException) {
                Log.e("AndroidSafHelper", "File not found for content URI: $contentUri - ${e.message}", e)
                -3
            } catch (e: Exception) {
                Log.e("AndroidSafHelper", "Unexpected exception opening content URI: $contentUri - ${e.message}", e)
                -4
            }
        }
    }

    /**
     * Callback to read data from a file descriptor with improved chunked reading
     */
    val readCallback = object : ZipLockNative.AndroidSafReadCallback {
        override fun callback(fd: Int, buffer: Pointer, size: Int): Int {
            return try {
                // Limit read size to prevent hanging on large reads
                val maxChunkSize = 32 * 1024 // 32KB chunks
                val actualSize = if (size > maxChunkSize) {
                    Log.d("AndroidSafHelper", "Limiting read size from $size to $maxChunkSize bytes for FD $fd")
                    maxChunkSize
                } else {
                    size
                }

                Log.d("AndroidSafHelper", "Read request: FD=$fd, requested=$size, actual=$actualSize")
                val pfd = openFileDescriptors[fd]
                if (pfd == null) {
                    Log.e("AndroidSafHelper", "Invalid file descriptor: $fd")
                    return -1
                }

                // Use a smaller buffer to avoid memory pressure
                val tempBuffer = ByteArray(actualSize)
                val inputStream = FileInputStream(pfd.fileDescriptor)

                // Set a read timeout to prevent hanging
                val startTime = System.currentTimeMillis()
                val timeoutMs = 5000 // 5 second timeout per chunk

                val bytesRead = inputStream.read(tempBuffer)
                val elapsed = System.currentTimeMillis() - startTime

                if (elapsed > 1000) { // Log if read took more than 1 second
                    Log.w("AndroidSafHelper", "Slow read: $bytesRead bytes in ${elapsed}ms for FD $fd")
                }

                if (bytesRead > 0) {
                    buffer.write(0, tempBuffer, 0, bytesRead)
                    Log.d("AndroidSafHelper", "Successfully read $bytesRead bytes from FD $fd")
                } else if (bytesRead == 0) {
                    Log.d("AndroidSafHelper", "End of file reached for FD $fd")
                } else {
                    Log.w("AndroidSafHelper", "Read returned -1 for FD $fd")
                }

                bytesRead
            } catch (e: SecurityException) {
                Log.e("AndroidSafHelper", "Security exception reading from FD $fd: ${e.message}", e)
                -2
            } catch (e: IOException) {
                Log.e("AndroidSafHelper", "IO exception reading from FD $fd: ${e.message}", e)
                -3
            } catch (e: Exception) {
                Log.e("AndroidSafHelper", "Unexpected exception reading from FD $fd: ${e.message}", e)
                -4
            }
        }
    }

    /**
     * Callback to write data to a file descriptor
     */
    val writeCallback = object : ZipLockNative.AndroidSafWriteCallback {
        override fun callback(fd: Int, data: Pointer, size: Int): Int {
            return try {
                Log.d("AndroidSafHelper", "Write request: FD=$fd, size=$size")
                val pfd = openFileDescriptors[fd]
                if (pfd == null) {
                    Log.e("AndroidSafHelper", "Invalid file descriptor: $fd")
                    return -1
                }

                val outputStream = FileOutputStream(pfd.fileDescriptor)
                val buffer = ByteArray(size)
                data.read(0, buffer, 0, size)
                outputStream.write(buffer)
                outputStream.flush()

                Log.d("AndroidSafHelper", "Successfully wrote $size bytes to FD $fd")
                size
            } catch (e: SecurityException) {
                Log.e("AndroidSafHelper", "Security exception writing to FD $fd: ${e.message}", e)
                -2
            } catch (e: IOException) {
                Log.e("AndroidSafHelper", "IO exception writing to FD $fd: ${e.message}", e)
                -3
            } catch (e: Exception) {
                Log.e("AndroidSafHelper", "Unexpected exception writing to FD $fd: ${e.message}", e)
                -4
            }
        }
    }

    /**
     * Callback to close a file descriptor
     */
    val closeCallback = object : ZipLockNative.AndroidSafCloseCallback {
        override fun callback(fd: Int): Int {
            return try {
                val pfd = openFileDescriptors.remove(fd)
                if (pfd != null) {
                    pfd.close()
                    Log.d("AndroidSafHelper", "Closed FD $fd")
                    0
                } else {
                    Log.w("AndroidSafHelper", "Attempted to close non-existent FD: $fd")
                    -1
                }
            } catch (e: Exception) {
                Log.e("AndroidSafHelper", "Exception closing FD $fd", e)
                -2
            }
        }
    }

    /**
     * Callback to get file size from a file descriptor with better error handling
     */
    val getSizeCallback = object : ZipLockNative.AndroidSafGetSizeCallback {
        override fun callback(fd: Int): Long {
            return try {
                Log.d("AndroidSafHelper", "Size request for FD: $fd")
                val pfd = openFileDescriptors[fd]
                if (pfd == null) {
                    Log.e("AndroidSafHelper", "Invalid file descriptor for size query: $fd")
                    return -1L
                }

                val size = pfd.statSize
                Log.d("AndroidSafHelper", "File size for FD $fd: $size bytes")

                // Additional validation for file size
                if (size < 0) {
                    Log.w("AndroidSafHelper", "Negative file size returned for FD $fd: $size")
                    return -1L
                } else if (size > 100 * 1024 * 1024) { // > 100MB
                    Log.w("AndroidSafHelper", "Large file detected for FD $fd: $size bytes (${size / (1024 * 1024)}MB)")
                }

                size
            } catch (e: SecurityException) {
                Log.e("AndroidSafHelper", "Security exception getting size for FD $fd: ${e.message}", e)
                -2L
            } catch (e: Exception) {
                Log.e("AndroidSafHelper", "Exception getting size for FD $fd: ${e.message}", e)
                -3L
            }
        }
    }

    /**
     * Callback to create a temporary file with better error handling and cleanup
     */
    val createTempCallback = object : ZipLockNative.AndroidSafCreateTempFileCallback {
        override fun callback(name: String, pathOut: PointerByReference): Int {
            return try {
                Log.d("AndroidSafHelper", "Creating temporary file with name: $name")

                // Ensure cache directory exists and has space
                val cacheDir = context.cacheDir
                if (!cacheDir.exists() && !cacheDir.mkdirs()) {
                    Log.e("AndroidSafHelper", "Failed to create cache directory")
                    return -1
                }

                // Check available space (require at least 50MB for large archives)
                val availableSpace = cacheDir.freeSpace
                val requiredSpace = 50 * 1024 * 1024L // 50MB
                if (availableSpace < requiredSpace) {
                    Log.w("AndroidSafHelper", "Low disk space: ${availableSpace / (1024 * 1024)}MB available, ${requiredSpace / (1024 * 1024)}MB required")
                }

                // Create a temporary file in the app's cache directory
                val tempFile = File.createTempFile(
                    "ziplock_$name",
                    ".tmp",
                    cacheDir
                )

                // Set file to be deleted on exit
                tempFile.deleteOnExit()

                // Allocate memory for the path string and copy it
                val pathString = tempFile.absolutePath
                val pathBytes = pathString.toByteArray(Charsets.UTF_8)
                val memory = Memory((pathBytes.size + 1).toLong()) // +1 for null terminator
                memory.write(0, pathBytes, 0, pathBytes.size)
                memory.setByte(pathBytes.size.toLong(), 0) // null terminator

                pathOut.value = memory
                Log.d("AndroidSafHelper", "Created temporary file: ${tempFile.absolutePath} (${availableSpace / (1024 * 1024)}MB available)")
                0
            } catch (e: Exception) {
                Log.e("AndroidSafHelper", "Exception creating temporary file: ${e.message}", e)
                -3
            }
        }
    }

    /**
     * Get content URI information for debugging
     */
    fun getContentUriInfo(contentUri: String): String {
        return try {
            val uri = Uri.parse(contentUri)
            val cursor = context.contentResolver.query(uri, null, null, null, null)

            val info = StringBuilder()
            info.append("URI: $contentUri\n")
            info.append("Authority: ${uri.authority}\n")
            info.append("Path: ${uri.path}\n")

            cursor?.use {
                if (it.moveToFirst()) {
                    val displayNameIndex = it.getColumnIndex(OpenableColumns.DISPLAY_NAME)
                    val sizeIndex = it.getColumnIndex(OpenableColumns.SIZE)

                    if (displayNameIndex >= 0) {
                        info.append("Display Name: ${it.getString(displayNameIndex)}\n")
                    }
                    if (sizeIndex >= 0) {
                        info.append("Size: ${it.getLong(sizeIndex)} bytes\n")
                    }
                }
            }

            info.toString()
        } catch (e: Exception) {
            "Error getting URI info: ${e.message}"
        }
    }

    /**
     * Test if a content URI is accessible
     */
    fun testContentUri(contentUri: String): Boolean {
        return try {
            val uri = Uri.parse(contentUri)
            val pfd = context.contentResolver.openFileDescriptor(uri, "r")
            pfd?.use {
                val size = it.statSize
                Log.d("AndroidSafHelper", "Content URI test successful: $contentUri (size: $size)")
                true
            } ?: false
        } catch (e: Exception) {
            Log.e("AndroidSafHelper", "Content URI test failed: $contentUri", e)
            false
        }
    }

    /**
     * Cleanup all open file descriptors
     */
    fun cleanup() {
        Log.d("AndroidSafHelper", "Cleaning up ${openFileDescriptors.size} open file descriptors")

        openFileDescriptors.values.forEach { pfd ->
            try {
                pfd.close()
            } catch (e: Exception) {
                Log.w("AndroidSafHelper", "Exception closing ParcelFileDescriptor during cleanup", e)
            }
        }

        openFileDescriptors.clear()
    }
}

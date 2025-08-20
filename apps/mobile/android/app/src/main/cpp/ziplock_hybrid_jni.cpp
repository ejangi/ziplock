#include <jni.h>
#include <string>
#include <cstring>
#include "ziplock_hybrid.h"

extern "C" {

// Helper function to convert Java string to C string
const char* jstring_to_cstring(JNIEnv* env, jstring jstr) {
    if (jstr == nullptr) return nullptr;
    return env->GetStringUTFChars(jstr, nullptr);
}

// Helper function to release C string
void release_cstring(JNIEnv* env, jstring jstr, const char* cstr) {
    if (jstr != nullptr && cstr != nullptr) {
        env->ReleaseStringUTFChars(jstr, cstr);
    }
}

// Helper function to create Java string from C string
jstring cstring_to_jstring(JNIEnv* env, const char* cstr) {
    if (cstr == nullptr) return nullptr;
    jstring result = env->NewStringUTF(cstr);
    ziplock_hybrid_string_free(const_cast<char*>(cstr));
    return result;
}

//
// Library Management Functions
//

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridInit(JNIEnv* env, jobject thiz) {
    return ziplock_hybrid_init();
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridGetVersion(JNIEnv* env, jobject thiz) {
    char* version = ziplock_hybrid_get_version();
    return cstring_to_jstring(env, version);
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridGetLastError(JNIEnv* env, jobject thiz) {
    char* error = ziplock_hybrid_get_last_error();
    return cstring_to_jstring(env, error);
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCleanup(JNIEnv* env, jobject thiz) {
    return ziplock_hybrid_cleanup();
}

//
// Memory Management Functions
//

JNIEXPORT void JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridStringFree(JNIEnv* env, jobject thiz, jlong ptr) {
    if (ptr != 0) {
        ziplock_hybrid_string_free(reinterpret_cast<char*>(ptr));
    }
}

JNIEXPORT void JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialFree(JNIEnv* env, jobject thiz, jlong credential_id) {
    ziplock_hybrid_credential_free(static_cast<uint64_t>(credential_id));
}

//
// Credential Management Functions
//

JNIEXPORT jlong JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialNew(JNIEnv* env, jobject thiz, jstring title, jstring type) {
    const char* title_cstr = jstring_to_cstring(env, title);
    const char* type_cstr = jstring_to_cstring(env, type);
    
    uint64_t result = ziplock_hybrid_credential_new(title_cstr, type_cstr);
    
    release_cstring(env, title, title_cstr);
    release_cstring(env, type, type_cstr);
    
    return static_cast<jlong>(result);
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialAddField(
    JNIEnv* env, jobject thiz, 
    jlong credential_id, 
    jstring name, 
    jint field_type, 
    jstring value, 
    jstring label, 
    jint sensitive
) {
    const char* name_cstr = jstring_to_cstring(env, name);
    const char* value_cstr = jstring_to_cstring(env, value);
    const char* label_cstr = label != nullptr ? jstring_to_cstring(env, label) : nullptr;
    
    int result = ziplock_hybrid_credential_add_field(
        static_cast<uint64_t>(credential_id),
        name_cstr,
        static_cast<int>(field_type),
        value_cstr,
        label_cstr,
        static_cast<int>(sensitive)
    );
    
    release_cstring(env, name, name_cstr);
    release_cstring(env, value, value_cstr);
    if (label != nullptr) {
        release_cstring(env, label, label_cstr);
    }
    
    return static_cast<jint>(result);
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialGetField(
    JNIEnv* env, jobject thiz, 
    jlong credential_id, 
    jstring field_name
) {
    const char* name_cstr = jstring_to_cstring(env, field_name);
    
    char* field_value = ziplock_hybrid_credential_get_field(
        static_cast<uint64_t>(credential_id),
        name_cstr
    );
    
    release_cstring(env, field_name, name_cstr);
    
    return cstring_to_jstring(env, field_value);
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialToJson(JNIEnv* env, jobject thiz, jlong credential_id) {
    char* json = ziplock_hybrid_credential_to_json(static_cast<uint64_t>(credential_id));
    return cstring_to_jstring(env, json);
}

JNIEXPORT jlong JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialFromJson(JNIEnv* env, jobject thiz, jstring json) {
    const char* json_cstr = jstring_to_cstring(env, json);
    
    uint64_t result = ziplock_hybrid_credential_from_json(json_cstr);
    
    release_cstring(env, json, json_cstr);
    
    return static_cast<jlong>(result);
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridCredentialValidate(JNIEnv* env, jobject thiz, jlong credential_id) {
    return ziplock_hybrid_credential_validate(static_cast<uint64_t>(credential_id));
}

//
// Password Functions
//

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridPasswordGenerate(
    JNIEnv* env, jobject thiz, 
    jint length, 
    jint uppercase, 
    jint lowercase, 
    jint numbers, 
    jint symbols
) {
    char* password = ziplock_hybrid_password_generate(
        static_cast<int>(length),
        static_cast<int>(uppercase),
        static_cast<int>(lowercase),
        static_cast<int>(numbers),
        static_cast<int>(symbols)
    );
    
    return cstring_to_jstring(env, password);
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridPasswordStrength(JNIEnv* env, jobject thiz, jstring password) {
    const char* password_cstr = jstring_to_cstring(env, password);
    
    int result = ziplock_hybrid_password_strength(password_cstr);
    
    release_cstring(env, password, password_cstr);
    
    return static_cast<jint>(result);
}

JNIEXPORT jdouble JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridPasswordEntropy(JNIEnv* env, jobject thiz, jstring password) {
    const char* password_cstr = jstring_to_cstring(env, password);
    
    double result = ziplock_hybrid_password_entropy(password_cstr);
    
    release_cstring(env, password, password_cstr);
    
    return static_cast<jdouble>(result);
}

//
// Validation Functions
//

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridEmailValidate(JNIEnv* env, jobject thiz, jstring email) {
    const char* email_cstr = jstring_to_cstring(env, email);
    
    int result = ziplock_hybrid_email_validate(email_cstr);
    
    release_cstring(env, email, email_cstr);
    
    return static_cast<jint>(result);
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridUrlValidate(JNIEnv* env, jobject thiz, jstring url) {
    const char* url_cstr = jstring_to_cstring(env, url);
    
    int result = ziplock_hybrid_url_validate(url_cstr);
    
    release_cstring(env, url, url_cstr);
    
    return static_cast<jint>(result);
}

JNIEXPORT jint JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridPhoneValidate(JNIEnv* env, jobject thiz, jstring phone, jstring country_code) {
    const char* phone_cstr = jstring_to_cstring(env, phone);
    const char* country_cstr = country_code != nullptr ? jstring_to_cstring(env, country_code) : nullptr;
    
    int result = ziplock_hybrid_phone_validate(phone_cstr, country_cstr);
    
    release_cstring(env, phone, phone_cstr);
    if (country_code != nullptr) {
        release_cstring(env, country_code, country_cstr);
    }
    
    return static_cast<jint>(result);
}

//
// Cryptographic Functions
//

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridEncryptData(JNIEnv* env, jobject thiz, jstring data, jstring password) {
    const char* data_cstr = jstring_to_cstring(env, data);
    const char* password_cstr = jstring_to_cstring(env, password);
    
    char* encrypted = ziplock_hybrid_encrypt_data(data_cstr, password_cstr);
    
    release_cstring(env, data, data_cstr);
    release_cstring(env, password, password_cstr);
    
    return cstring_to_jstring(env, encrypted);
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridDecryptData(JNIEnv* env, jobject thiz, jstring encrypted_data, jstring password) {
    const char* encrypted_cstr = jstring_to_cstring(env, encrypted_data);
    const char* password_cstr = jstring_to_cstring(env, password);
    
    char* decrypted = ziplock_hybrid_decrypt_data(encrypted_cstr, password_cstr);
    
    release_cstring(env, encrypted_data, encrypted_cstr);
    release_cstring(env, password, password_cstr);
    
    return cstring_to_jstring(env, decrypted);
}

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridGenerateSalt(JNIEnv* env, jobject thiz) {
    char* salt = ziplock_hybrid_generate_salt();
    return cstring_to_jstring(env, salt);
}

//
// Utility Functions
//

JNIEXPORT jstring JNICALL
Java_com_ziplock_ffi_ZipLockDataManager_hybridTestEcho(JNIEnv* env, jobject thiz, jstring input) {
    const char* input_cstr = jstring_to_cstring(env, input);
    
    char* echo = ziplock_hybrid_test_echo(input_cstr);
    
    release_cstring(env, input, input_cstr);
    
    return cstring_to_jstring(env, echo);
}

} // extern "C"
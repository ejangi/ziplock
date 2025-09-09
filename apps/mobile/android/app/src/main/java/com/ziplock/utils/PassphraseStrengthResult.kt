package com.ziplock.utils

import kotlinx.serialization.Serializable

/**
 * PassphraseStrengthResult - Passphrase Strength Analysis
 *
 * This class provides passphrase strength analysis functionality
 * for the ZipLock application. It evaluates passphrase quality
 * based on various criteria and provides feedback to users.
 */
@Serializable
data class PassphraseStrengthResult(
    val score: Int, // 0-100 strength score
    val level: StrengthLevel,
    val isValid: Boolean,
    val feedback: List<String> = emptyList(),
    val estimatedCrackTime: String = "",
    val entropy: Double = 0.0
) {
    enum class StrengthLevel {
        VERY_WEAK,
        WEAK,
        FAIR,
        GOOD,
        STRONG,
        VERY_STRONG
    }

    companion object {
        /**
         * Analyze passphrase strength
         */
        fun analyze(passphrase: String): PassphraseStrengthResult {
            if (passphrase.isEmpty()) {
                return PassphraseStrengthResult(
                    score = 0,
                    level = StrengthLevel.VERY_WEAK,
                    isValid = false,
                    feedback = listOf("Passphrase cannot be empty"),
                    estimatedCrackTime = "Instantly"
                )
            }

            var score = 0
            val feedback = mutableListOf<String>()

            // Length analysis
            when {
                passphrase.length >= 16 -> {
                    score += 30
                    feedback.add("✓ Excellent length")
                }
                passphrase.length >= 12 -> {
                    score += 25
                    feedback.add("✓ Good length")
                }
                passphrase.length >= 8 -> {
                    score += 15
                    feedback.add("✓ Adequate length")
                }
                else -> {
                    score += 5
                    feedback.add("⚠ Use at least 8 characters")
                }
            }

            // Character variety
            var characterTypes = 0
            if (passphrase.any { it.isLowerCase() }) {
                characterTypes++
                score += 10
            } else {
                feedback.add("⚠ Add lowercase letters")
            }

            if (passphrase.any { it.isUpperCase() }) {
                characterTypes++
                score += 10
            } else {
                feedback.add("⚠ Add uppercase letters")
            }

            if (passphrase.any { it.isDigit() }) {
                characterTypes++
                score += 10
            } else {
                feedback.add("⚠ Add numbers")
            }

            if (passphrase.any { !it.isLetterOrDigit() }) {
                characterTypes++
                score += 15
            } else {
                feedback.add("⚠ Add special characters")
            }

            // Bonus for high character variety
            if (characterTypes >= 4) {
                score += 15
                feedback.add("✓ Great character variety")
            }

            // Check for common patterns
            if (containsCommonPatterns(passphrase)) {
                score -= 20
                feedback.add("⚠ Avoid common patterns")
            }

            // Check for repeated characters
            if (hasRepeatedCharacters(passphrase)) {
                score -= 10
                feedback.add("⚠ Avoid repeated characters")
            }

            // Check for dictionary words (simplified)
            if (containsCommonWords(passphrase)) {
                score -= 15
                feedback.add("⚠ Avoid common words")
            }

            // Ensure score is within bounds
            score = score.coerceIn(0, 100)

            val level = when (score) {
                in 0..20 -> StrengthLevel.VERY_WEAK
                in 21..40 -> StrengthLevel.WEAK
                in 41..60 -> StrengthLevel.FAIR
                in 61..75 -> StrengthLevel.GOOD
                in 76..90 -> StrengthLevel.STRONG
                else -> StrengthLevel.VERY_STRONG
            }

            val isValid = score >= 40 && passphrase.length >= 8

            val crackTime = estimateCrackTime(score, passphrase.length)
            val entropy = calculateEntropy(passphrase)

            return PassphraseStrengthResult(
                score = score,
                level = level,
                isValid = isValid,
                feedback = feedback,
                estimatedCrackTime = crackTime,
                entropy = entropy
            )
        }

        private fun containsCommonPatterns(passphrase: String): Boolean {
            val commonPatterns = listOf(
                "123", "abc", "qwe", "asd", "zxc", "000", "111", "222",
                "password", "admin", "user", "test", "login"
            )
            return commonPatterns.any { pattern ->
                passphrase.lowercase().contains(pattern)
            }
        }

        private fun hasRepeatedCharacters(passphrase: String): Boolean {
            var consecutiveCount = 1
            for (i in 1 until passphrase.length) {
                if (passphrase[i] == passphrase[i - 1]) {
                    consecutiveCount++
                    if (consecutiveCount >= 3) return true
                } else {
                    consecutiveCount = 1
                }
            }
            return false
        }

        private fun containsCommonWords(passphrase: String): Boolean {
            val commonWords = listOf(
                "password", "admin", "user", "test", "login", "welcome",
                "hello", "world", "secret", "private", "secure", "safe",
                "home", "work", "office", "computer", "mobile", "phone"
            )
            return commonWords.any { word ->
                passphrase.lowercase().contains(word)
            }
        }

        private fun estimateCrackTime(score: Int, length: Int): String {
            return when (score) {
                in 0..20 -> "Minutes"
                in 21..40 -> "Hours"
                in 41..60 -> "Days"
                in 61..75 -> "Months"
                in 76..90 -> "Years"
                else -> "Centuries"
            }
        }

        private fun calculateEntropy(passphrase: String): Double {
            val charSet = mutableSetOf<Char>()
            passphrase.forEach { charSet.add(it) }

            val alphabetSize = when {
                charSet.any { !it.isLetterOrDigit() } &&
                charSet.any { it.isUpperCase() } &&
                charSet.any { it.isLowerCase() } &&
                charSet.any { it.isDigit() } -> 95 // Full ASCII printable

                charSet.any { it.isUpperCase() } &&
                charSet.any { it.isLowerCase() } &&
                charSet.any { it.isDigit() } -> 62 // Upper + Lower + Digits

                charSet.any { it.isLetter() } &&
                charSet.any { it.isDigit() } -> 36 // Letters + Digits

                charSet.any { it.isLetter() } -> 26 // Letters only
                charSet.any { it.isDigit() } -> 10 // Digits only
                else -> 1
            }

            return passphrase.length * Math.log(alphabetSize.toDouble()) / Math.log(2.0)
        }
    }

    /**
     * Get color for UI representation
     */
    fun getColor(): Int {
        return when (level) {
            StrengthLevel.VERY_WEAK -> 0xFFE53E3E.toInt() // Red
            StrengthLevel.WEAK -> 0xFFFF8A00.toInt() // Orange
            StrengthLevel.FAIR -> 0xFFFFC107.toInt() // Yellow
            StrengthLevel.GOOD -> 0xFF38A169.toInt() // Green
            StrengthLevel.STRONG -> 0xFF00A86B.toInt() // Dark Green
            StrengthLevel.VERY_STRONG -> 0xFF1A365D.toInt() // Dark Blue
        }
    }

    /**
     * Get display text for strength level
     */
    fun getLevelText(): String {
        return when (level) {
            StrengthLevel.VERY_WEAK -> "Very Weak"
            StrengthLevel.WEAK -> "Weak"
            StrengthLevel.FAIR -> "Fair"
            StrengthLevel.GOOD -> "Good"
            StrengthLevel.STRONG -> "Strong"
            StrengthLevel.VERY_STRONG -> "Very Strong"
        }
    }

    /**
     * Get progress value for progress indicators (0.0 to 1.0)
     */
    fun getProgress(): Float {
        return score / 100.0f
    }
}

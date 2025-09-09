//! Password generation and validation utilities
//!
//! This module provides secure password generation, strength assessment,
//! and validation utilities for the ZipLock password manager.

use rand::{thread_rng, Rng, RngCore};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

/// Password character sets for generation
pub struct CharacterSets;

impl CharacterSets {
    pub const LOWERCASE: &'static str = "abcdefghijklmnopqrstuvwxyz";
    pub const UPPERCASE: &'static str = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    pub const DIGITS: &'static str = "0123456789";
    pub const SYMBOLS: &'static str = "!@#$%^&*()_+-=[]{}|;:,.<>?";
    pub const AMBIGUOUS: &'static str = "0O1lI";
}

/// Password generation options
#[derive(Debug, Clone)]
pub struct PasswordOptions {
    /// Length of the password to generate
    pub length: usize,
    /// Include lowercase letters
    pub include_lowercase: bool,
    /// Include uppercase letters
    pub include_uppercase: bool,
    /// Include digits
    pub include_digits: bool,
    /// Include symbols
    pub include_symbols: bool,
    /// Exclude ambiguous characters (0, O, 1, l, I)
    pub exclude_ambiguous: bool,
    /// Custom character set (overrides other settings if provided)
    pub custom_charset: Option<String>,
}

impl Default for PasswordOptions {
    fn default() -> Self {
        Self {
            length: 16,
            include_lowercase: true,
            include_uppercase: true,
            include_digits: true,
            include_symbols: true,
            exclude_ambiguous: false,
            custom_charset: None,
        }
    }
}

/// Password strength levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordStrength {
    VeryWeak,
    Weak,
    Fair,
    Good,
    Strong,
    VeryStrong,
}

impl PasswordStrength {
    /// Get a descriptive name for the strength level
    pub fn name(&self) -> &'static str {
        match self {
            PasswordStrength::VeryWeak => "Very Weak",
            PasswordStrength::Weak => "Weak",
            PasswordStrength::Fair => "Fair",
            PasswordStrength::Good => "Good",
            PasswordStrength::Strong => "Strong",
            PasswordStrength::VeryStrong => "Very Strong",
        }
    }

    /// Get a score from 0-100 for the strength level
    pub fn score(&self) -> u8 {
        match self {
            PasswordStrength::VeryWeak => 10,
            PasswordStrength::Weak => 25,
            PasswordStrength::Fair => 50,
            PasswordStrength::Good => 75,
            PasswordStrength::Strong => 90,
            PasswordStrength::VeryStrong => 100,
        }
    }
}

/// Password analysis result
#[derive(Debug, Clone)]
pub struct PasswordAnalysis {
    /// Overall strength assessment
    pub strength: PasswordStrength,
    /// Detailed score (0-100)
    pub score: u8,
    /// Estimated entropy in bits
    pub entropy: f64,
    /// Whether password appears in common password lists
    pub is_common: bool,
    /// Character set diversity score
    pub diversity: u8,
    /// Feedback messages for improvement
    pub feedback: Vec<String>,
}

/// Password generator
pub struct PasswordGenerator;

impl PasswordGenerator {
    /// Generate a secure password with the given options
    pub fn generate(options: &PasswordOptions) -> Result<String, &'static str> {
        if options.length == 0 {
            return Err("Password length must be greater than 0");
        }

        let charset = if let Some(ref custom) = options.custom_charset {
            custom.clone()
        } else {
            Self::build_charset(options)
        };

        if charset.is_empty() {
            return Err("Character set is empty - enable at least one character type");
        }

        let mut rng = thread_rng();
        let charset_chars: Vec<char> = charset.chars().collect();

        let password: String = (0..options.length)
            .map(|_| {
                let idx = rng.gen_range(0..charset_chars.len());
                charset_chars[idx]
            })
            .collect();

        Ok(password)
    }

    /// Generate a passphrase using word lists
    pub fn generate_passphrase(word_count: usize, separator: &str) -> Result<String, &'static str> {
        if word_count == 0 {
            return Err("Word count must be greater than 0");
        }

        // Simple word list - in a real implementation, this would come from a larger dictionary
        let words = [
            "apple",
            "beach",
            "cloud",
            "dance",
            "eagle",
            "flame",
            "grace",
            "house",
            "island",
            "jungle",
            "kite",
            "lemon",
            "mountain",
            "ocean",
            "piano",
            "quiet",
            "river",
            "sunset",
            "tiger",
            "umbrella",
            "valley",
            "whale",
            "xylophone",
            "yacht",
            "zebra",
            "anchor",
            "bridge",
            "castle",
            "dragon",
            "forest",
            "guitar",
            "harmony",
            "ivory",
            "jewel",
            "knight",
            "lighthouse",
            "melody",
            "nature",
            "orchid",
            "phoenix",
            "quartz",
            "rainbow",
            "serenity",
            "thunder",
            "unicorn",
            "violet",
            "wisdom",
            "crystal",
            "yonder",
        ];

        let mut rng = thread_rng();
        let selected_words: Vec<&str> = (0..word_count)
            .map(|_| {
                let idx = rng.gen_range(0..words.len());
                words[idx]
            })
            .collect();

        Ok(selected_words.join(separator))
    }

    /// Build character set based on options
    fn build_charset(options: &PasswordOptions) -> String {
        let mut charset = String::new();

        if options.include_lowercase {
            charset.push_str(CharacterSets::LOWERCASE);
        }
        if options.include_uppercase {
            charset.push_str(CharacterSets::UPPERCASE);
        }
        if options.include_digits {
            charset.push_str(CharacterSets::DIGITS);
        }
        if options.include_symbols {
            charset.push_str(CharacterSets::SYMBOLS);
        }

        if options.exclude_ambiguous {
            charset.retain(|c| !CharacterSets::AMBIGUOUS.contains(c));
        }

        charset
    }
}

/// Password analyzer for strength assessment
pub struct PasswordAnalyzer;

impl PasswordAnalyzer {
    /// Analyze password strength and provide feedback
    pub fn analyze(password: &str) -> PasswordAnalysis {
        let mut score = 0u8;
        let mut feedback = Vec::new();

        // Length scoring
        let length_score = Self::score_length(password, &mut feedback);
        score = score.saturating_add(length_score);

        // Character diversity scoring
        let diversity_score = Self::score_diversity(password, &mut feedback);
        score = score.saturating_add(diversity_score);

        // Pattern scoring
        let pattern_score = Self::score_patterns(password, &mut feedback);
        score = score.saturating_add(pattern_score);

        // Common password check
        let is_common = Self::is_common_password(password);
        if is_common {
            score = score.saturating_sub(30);
            feedback.push("This appears to be a common password".to_string());
        }

        // Calculate entropy
        let entropy = Self::calculate_entropy(password);

        // Determine strength level
        let strength = match score {
            0..=20 => PasswordStrength::VeryWeak,
            21..=40 => PasswordStrength::Weak,
            41..=60 => PasswordStrength::Fair,
            61..=80 => PasswordStrength::Good,
            81..=95 => PasswordStrength::Strong,
            96..=100 => PasswordStrength::VeryStrong,
            _ => PasswordStrength::VeryStrong,
        };

        PasswordAnalysis {
            strength,
            score,
            entropy,
            is_common,
            diversity: diversity_score,
            feedback,
        }
    }

    /// Score password based on length
    fn score_length(password: &str, feedback: &mut Vec<String>) -> u8 {
        let len = password.len();
        match len {
            0..=4 => {
                feedback.push("Password is too short - use at least 8 characters".to_string());
                5
            }
            5..=7 => {
                feedback
                    .push("Password is short - consider using at least 12 characters".to_string());
                15
            }
            8..=11 => 25,
            12..=15 => 30,
            16..=20 => 35,
            _ => 40,
        }
    }

    /// Score password based on character diversity
    fn score_diversity(password: &str, feedback: &mut Vec<String>) -> u8 {
        let mut score = 0u8;
        let mut has_lower = false;
        let mut has_upper = false;
        let mut has_digit = false;
        let mut has_symbol = false;

        for c in password.chars() {
            if c.is_ascii_lowercase() {
                has_lower = true;
            } else if c.is_ascii_uppercase() {
                has_upper = true;
            } else if c.is_ascii_digit() {
                has_digit = true;
            } else if !c.is_ascii_alphanumeric() {
                has_symbol = true;
            }
        }

        if has_lower {
            score += 10;
        } else {
            feedback.push("Add lowercase letters".to_string());
        }

        if has_upper {
            score += 10;
        } else {
            feedback.push("Add uppercase letters".to_string());
        }

        if has_digit {
            score += 10;
        } else {
            feedback.push("Add numbers".to_string());
        }

        if has_symbol {
            score += 15;
        } else {
            feedback.push("Add symbols (!@#$%^&*)".to_string());
        }

        score
    }

    /// Score password based on patterns (repetition, sequences, etc.)
    fn score_patterns(password: &str, feedback: &mut Vec<String>) -> u8 {
        let mut score = 15u8; // Start with full pattern score

        // Check for repeated characters
        let mut char_counts = std::collections::HashMap::new();
        for c in password.chars() {
            *char_counts.entry(c).or_insert(0) += 1;
        }

        let max_repeat = char_counts.values().max().unwrap_or(&1);
        if *max_repeat > password.len() / 3 {
            score = score.saturating_sub(10);
            feedback.push("Avoid repeating the same character too often".to_string());
        }

        // Check for sequential characters (abc, 123)
        let chars: Vec<char> = password.chars().collect();
        let mut sequential_count = 0;
        for window in chars.windows(3) {
            if window[1] as u8 == window[0] as u8 + 1 && window[2] as u8 == window[1] as u8 + 1 {
                sequential_count += 1;
            }
        }

        if sequential_count > 0 {
            score = score.saturating_sub(5);
            feedback.push("Avoid sequential characters (abc, 123)".to_string());
        }

        score
    }

    /// Check if password is commonly used (simplified check)
    fn is_common_password(password: &str) -> bool {
        let common_passwords = [
            "password",
            "123456",
            "123456789",
            "12345678",
            "12345",
            "1234567",
            "123",
            "password123",
            "admin",
            "qwerty",
            "abc123",
            "Password1",
            "welcome",
            "monkey",
            "dragon",
            "letmein",
            "trustno1",
            "sunshine",
            "master",
            "hello",
            "freedom",
            "whatever",
            "qazwsx",
            "123321",
            "654321",
        ];

        let lower_password = password.to_lowercase();
        common_passwords.contains(&lower_password.as_str())
    }

    /// Calculate password entropy in bits
    fn calculate_entropy(password: &str) -> f64 {
        if password.is_empty() {
            return 0.0;
        }

        let mut charset_size = 0;
        let chars: HashSet<char> = password.chars().collect();

        // Estimate character set size based on characters used
        let has_lower = chars.iter().any(|c| c.is_ascii_lowercase());
        let has_upper = chars.iter().any(|c| c.is_ascii_uppercase());
        let has_digit = chars.iter().any(|c| c.is_ascii_digit());
        let has_symbol = chars.iter().any(|c| !c.is_ascii_alphanumeric());

        if has_lower {
            charset_size += 26;
        }
        if has_upper {
            charset_size += 26;
        }
        if has_digit {
            charset_size += 10;
        }
        if has_symbol {
            charset_size += 32; // Approximate
        }

        if charset_size == 0 {
            charset_size = 1; // Avoid log(0)
        }

        password.len() as f64 * (charset_size as f64).log2()
    }
}

/// Utility functions for password operations
pub struct PasswordUtils;

impl PasswordUtils {
    /// Generate a secure random salt for password hashing
    pub fn generate_salt() -> [u8; 32] {
        let mut salt = [0u8; 32];
        thread_rng().fill_bytes(&mut salt);
        salt
    }

    /// Hash a password with salt using SHA-256 (for demonstration - use proper password hashing in production)
    pub fn hash_password(password: &str, salt: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.update(salt);
        format!("{:x}", hasher.finalize())
    }

    /// Check if two passwords are similar (for preventing similar password reuse)
    pub fn are_similar(password1: &str, password2: &str, threshold: f64) -> bool {
        let similarity = Self::calculate_similarity(password1, password2);
        similarity > threshold
    }

    /// Calculate Levenshtein distance-based similarity between two passwords
    fn calculate_similarity(s1: &str, s2: &str) -> f64 {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 && len2 == 0 {
            return 1.0;
        }

        let max_len = len1.max(len2);
        if max_len == 0 {
            return 1.0;
        }

        let distance = Self::levenshtein_distance(s1, s2);
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_generation() {
        let options = PasswordOptions::default();
        let password = PasswordGenerator::generate(&options).unwrap();

        assert_eq!(password.len(), options.length);
        assert!(!password.is_empty());
    }

    #[test]
    fn test_password_generation_options() {
        let options = PasswordOptions {
            length: 10,
            include_lowercase: true,
            include_uppercase: false,
            include_digits: false,
            include_symbols: false,
            exclude_ambiguous: false,
            custom_charset: None,
        };

        let password = PasswordGenerator::generate(&options).unwrap();
        assert_eq!(password.len(), 10);
        assert!(password.chars().all(|c| c.is_ascii_lowercase()));
    }

    #[test]
    fn test_passphrase_generation() {
        let passphrase = PasswordGenerator::generate_passphrase(4, "-").unwrap();
        let words: Vec<&str> = passphrase.split('-').collect();
        assert_eq!(words.len(), 4);
    }

    #[test]
    fn test_password_analysis() {
        let weak_password = "123";
        let analysis = PasswordAnalyzer::analyze(weak_password);
        assert_eq!(analysis.strength, PasswordStrength::VeryWeak);
        assert!(!analysis.feedback.is_empty());

        let strong_password = "MySecure!Password123";
        let analysis = PasswordAnalyzer::analyze(strong_password);
        assert!(matches!(
            analysis.strength,
            PasswordStrength::Good | PasswordStrength::Strong | PasswordStrength::VeryStrong
        ));
    }

    #[test]
    fn test_common_password_detection() {
        assert!(PasswordAnalyzer::is_common_password("password"));
        assert!(PasswordAnalyzer::is_common_password("123456"));
        assert!(!PasswordAnalyzer::is_common_password(
            "MyUniquePassword!123"
        ));
    }

    #[test]
    fn test_entropy_calculation() {
        let entropy1 = PasswordAnalyzer::calculate_entropy("abc");
        let entropy2 = PasswordAnalyzer::calculate_entropy("AbC123!@#");
        assert!(entropy2 > entropy1);
    }

    #[test]
    fn test_password_similarity() {
        assert!(PasswordUtils::are_similar("password1", "password2", 0.8));
        assert!(!PasswordUtils::are_similar(
            "password1",
            "completely_different",
            0.8
        ));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(PasswordUtils::levenshtein_distance("", ""), 0);
        assert_eq!(PasswordUtils::levenshtein_distance("abc", "abc"), 0);
        assert_eq!(PasswordUtils::levenshtein_distance("abc", "def"), 3);
        assert_eq!(PasswordUtils::levenshtein_distance("abc", "ab"), 1);
    }

    #[test]
    fn test_salt_generation() {
        let salt1 = PasswordUtils::generate_salt();
        let salt2 = PasswordUtils::generate_salt();
        assert_ne!(salt1, salt2); // Should be different
        assert_eq!(salt1.len(), 32);
    }

    #[test]
    fn test_password_hashing() {
        let password = "test_password";
        let salt = PasswordUtils::generate_salt();
        let hash1 = PasswordUtils::hash_password(password, &salt);
        let hash2 = PasswordUtils::hash_password(password, &salt);

        assert_eq!(hash1, hash2); // Same input should give same hash
        assert_eq!(hash1.len(), 64); // SHA-256 hex string length
    }

    #[test]
    fn test_empty_charset_error() {
        let options = PasswordOptions {
            length: 10,
            include_lowercase: false,
            include_uppercase: false,
            include_digits: false,
            include_symbols: false,
            exclude_ambiguous: false,
            custom_charset: None,
        };

        let result = PasswordGenerator::generate(&options);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_length_error() {
        let options = PasswordOptions {
            length: 0,
            ..Default::default()
        };

        let result = PasswordGenerator::generate(&options);
        assert!(result.is_err());
    }
}

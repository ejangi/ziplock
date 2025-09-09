//! Search and filtering utilities for ZipLock credentials
//!
//! This module provides comprehensive search functionality for finding
//! credentials based on various criteria including text search, tags,
//! field values, and metadata.

use regex::{Regex, RegexBuilder};
use std::collections::{HashMap, HashSet};

use crate::models::{CredentialRecord, FieldType};

/// Search query with multiple criteria
#[derive(Debug, Clone, PartialEq)]
pub struct SearchQuery {
    /// Text to search for in titles and field values
    pub text: Option<String>,

    /// Tags that must be present (AND logic)
    pub required_tags: Vec<String>,

    /// Tags that can be present (OR logic)
    pub optional_tags: Vec<String>,

    /// Credential types to include
    pub credential_types: Vec<String>,

    /// Field type filters
    pub field_types: Vec<FieldType>,

    /// Search in sensitive fields
    pub include_sensitive: bool,

    /// Case sensitive search
    pub case_sensitive: bool,

    /// Use regex for text search
    pub use_regex: bool,

    /// Search in field values
    pub search_field_values: bool,

    /// Search in notes
    pub search_notes: bool,

    /// Favorite credentials only
    pub favorites_only: bool,

    /// Folder path filter
    pub folder_path: Option<String>,
}

impl Default for SearchQuery {
    fn default() -> Self {
        Self {
            text: None,
            required_tags: Vec::new(),
            optional_tags: Vec::new(),
            credential_types: Vec::new(),
            field_types: Vec::new(),
            include_sensitive: false,
            case_sensitive: false,
            use_regex: false,
            search_field_values: true,
            search_notes: true,
            favorites_only: false,
            folder_path: None,
        }
    }
}

/// Search result with ranking information
#[derive(Debug, Clone, PartialEq)]
pub struct SearchResult {
    /// The credential that matches
    pub credential: CredentialRecord,

    /// Search relevance score (0.0 to 1.0)
    pub score: f64,

    /// Matched locations for highlighting
    pub matches: Vec<SearchMatch>,
}

/// Information about where a search term was found
#[derive(Debug, Clone, PartialEq)]
pub struct SearchMatch {
    /// Location type (title, field, notes, etc.)
    pub location: MatchLocation,

    /// Field name if location is a field
    pub field_name: Option<String>,

    /// Start position of the match
    pub start: usize,

    /// End position of the match
    pub end: usize,

    /// Matched text
    pub matched_text: String,
}

/// Where a search match was found
#[derive(Debug, Clone, PartialEq)]
pub enum MatchLocation {
    Title,
    FieldValue,
    FieldLabel,
    Notes,
    Tag,
    CredentialType,
}

impl SearchQuery {
    /// Create a simple text search query
    pub fn text<S: Into<String>>(text: S) -> Self {
        Self {
            text: Some(text.into()),
            ..Default::default()
        }
    }

    /// Create a tag search query
    pub fn with_tags(tags: Vec<String>) -> Self {
        Self {
            required_tags: tags,
            ..Default::default()
        }
    }

    /// Add required tag
    pub fn require_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.required_tags.push(tag.into());
        self
    }

    /// Add optional tag
    pub fn optional_tag<S: Into<String>>(mut self, tag: S) -> Self {
        self.optional_tags.push(tag.into());
        self
    }

    /// Filter by credential type
    pub fn credential_type<S: Into<String>>(mut self, cred_type: S) -> Self {
        self.credential_types.push(cred_type.into());
        self
    }

    /// Filter by field type
    pub fn field_type(mut self, field_type: FieldType) -> Self {
        self.field_types.push(field_type);
        self
    }

    /// Include sensitive fields in search
    pub fn include_sensitive(mut self, include: bool) -> Self {
        self.include_sensitive = include;
        self
    }

    /// Use case sensitive search
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Use regex for text search
    pub fn with_regex(mut self, use_regex: bool) -> Self {
        self.use_regex = use_regex;
        self
    }

    /// Search only favorites
    pub fn favorites_only(mut self, favorites: bool) -> Self {
        self.favorites_only = favorites;
        self
    }

    /// Filter by folder path
    pub fn in_folder<S: Into<String>>(mut self, folder: S) -> Self {
        self.folder_path = Some(folder.into());
        self
    }
}

/// Search engine for credentials
pub struct CredentialSearchEngine;

impl CredentialSearchEngine {
    /// Search credentials with the given query
    pub fn search(
        credentials: &HashMap<String, CredentialRecord>,
        query: &SearchQuery,
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for credential in credentials.values() {
            if let Some(result) = Self::match_credential(credential, query) {
                results.push(result);
            }
        }

        // Sort by relevance score (highest first)
        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        results
    }

    /// Check if a credential matches the search query
    fn match_credential(
        credential: &CredentialRecord,
        query: &SearchQuery,
    ) -> Option<SearchResult> {
        let mut score = 0.0;
        let mut matches = Vec::new();

        // Filter by favorites
        if query.favorites_only && !credential.favorite {
            return None;
        }

        // Filter by folder path
        if let Some(folder) = &query.folder_path {
            if !credential
                .folder_path
                .as_ref()
                .map_or(false, |path| path.starts_with(folder))
            {
                return None;
            }
        }

        // Filter by credential types
        if !query.credential_types.is_empty() {
            if !query.credential_types.contains(&credential.credential_type) {
                return None;
            }
        }

        // Check required tags
        for required_tag in &query.required_tags {
            if !credential.has_tag(required_tag) {
                return None;
            }
        }

        // Check optional tags (at least one must match if specified)
        if !query.optional_tags.is_empty() {
            let has_optional_tag = query
                .optional_tags
                .iter()
                .any(|tag| credential.has_tag(tag));
            if !has_optional_tag {
                return None;
            }
        }

        // Filter by field types
        if !query.field_types.is_empty() {
            let has_field_type = credential
                .fields
                .values()
                .any(|field| query.field_types.contains(&field.field_type));
            if !has_field_type {
                return None;
            }
        }

        // Text search
        if let Some(search_text) = &query.text {
            let (text_score, text_matches) =
                Self::search_text_in_credential(credential, search_text, query);

            if text_score == 0.0 {
                return None; // No text match found
            }

            score += text_score;
            matches.extend(text_matches);
        } else {
            score = 1.0; // Base score when no text search
        }

        // Bonus scoring
        score += Self::calculate_bonus_score(credential, query);

        Some(SearchResult {
            credential: credential.clone(),
            score,
            matches,
        })
    }

    /// Search for text within a credential
    fn search_text_in_credential(
        credential: &CredentialRecord,
        search_text: &str,
        query: &SearchQuery,
    ) -> (f64, Vec<SearchMatch>) {
        let mut total_score = 0.0;
        let mut matches = Vec::new();

        // Search in title (highest weight)
        if let Some((score, title_matches)) = Self::search_in_text(
            &credential.title,
            search_text,
            query,
            MatchLocation::Title,
            None,
        ) {
            total_score += score * 3.0; // Title matches are most important
            matches.extend(title_matches);
        }

        // Search in credential type
        if let Some((score, type_matches)) = Self::search_in_text(
            &credential.credential_type,
            search_text,
            query,
            MatchLocation::CredentialType,
            None,
        ) {
            total_score += score * 1.5;
            matches.extend(type_matches);
        }

        // Search in tags
        for tag in &credential.tags {
            if let Some((score, tag_matches)) =
                Self::search_in_text(tag, search_text, query, MatchLocation::Tag, None)
            {
                total_score += score * 2.0; // Tags are important
                matches.extend(tag_matches);
            }
        }

        // Search in field values and labels
        if query.search_field_values {
            for (field_name, field) in &credential.fields {
                // Skip sensitive fields if not included
                if field.sensitive && !query.include_sensitive {
                    continue;
                }

                // Search in field value
                if let Some((score, field_matches)) = Self::search_in_text(
                    &field.value,
                    search_text,
                    query,
                    MatchLocation::FieldValue,
                    Some(field_name.clone()),
                ) {
                    total_score += score;
                    matches.extend(field_matches);
                }

                // Search in field label
                if let Some(label) = &field.label {
                    if let Some((score, label_matches)) = Self::search_in_text(
                        label,
                        search_text,
                        query,
                        MatchLocation::FieldLabel,
                        Some(field_name.clone()),
                    ) {
                        total_score += score * 1.2; // Labels are slightly more important
                        matches.extend(label_matches);
                    }
                }
            }
        }

        // Search in notes
        if query.search_notes
            && credential
                .notes
                .as_ref()
                .map_or(false, |notes| !notes.is_empty())
        {
            if let Some((score, notes_matches)) = Self::search_in_text(
                credential.notes.as_ref().unwrap(),
                search_text,
                query,
                MatchLocation::Notes,
                None,
            ) {
                total_score += score * 0.8; // Notes are less important
                matches.extend(notes_matches);
            }
        }

        (total_score, matches)
    }

    /// Search for text within a specific string
    fn search_in_text(
        text: &str,
        search_text: &str,
        query: &SearchQuery,
        location: MatchLocation,
        field_name: Option<String>,
    ) -> Option<(f64, Vec<SearchMatch>)> {
        if text.is_empty() || search_text.is_empty() {
            return None;
        }

        let mut matches = Vec::new();
        let score: f64;

        if query.use_regex {
            // Regex search
            let regex_result = if query.case_sensitive {
                Regex::new(search_text)
            } else {
                RegexBuilder::new(search_text)
                    .case_insensitive(true)
                    .build()
            };

            match regex_result {
                Ok(regex) => {
                    for mat in regex.find_iter(text) {
                        matches.push(SearchMatch {
                            location: location.clone(),
                            field_name: field_name.clone(),
                            start: mat.start(),
                            end: mat.end(),
                            matched_text: mat.as_str().to_string(),
                        });
                    }
                }
                Err(_) => return None, // Invalid regex
            }
        } else {
            // Simple text search
            let search_lower = if query.case_sensitive {
                search_text.to_string()
            } else {
                search_text.to_lowercase()
            };

            let text_to_search = if query.case_sensitive {
                text.to_string()
            } else {
                text.to_lowercase()
            };

            let mut start = 0;
            while let Some(pos) = text_to_search[start..].find(&search_lower) {
                let absolute_pos = start + pos;
                matches.push(SearchMatch {
                    location: location.clone(),
                    field_name: field_name.clone(),
                    start: absolute_pos,
                    end: absolute_pos + search_text.len(),
                    matched_text: text[absolute_pos..absolute_pos + search_text.len()].to_string(),
                });
                start = absolute_pos + 1;
            }
        }

        if matches.is_empty() {
            return None;
        }

        // Calculate score based on match quality
        score = Self::calculate_text_match_score(text, search_text, &matches);

        Some((score, matches))
    }

    /// Calculate score for text matches
    fn calculate_text_match_score(text: &str, search_text: &str, matches: &[SearchMatch]) -> f64 {
        if matches.is_empty() {
            return 0.0;
        }

        let text_len = text.len() as f64;
        let search_len = search_text.len() as f64;
        let match_count = matches.len() as f64;

        // Base score from match ratio
        let coverage = (search_len * match_count) / text_len;

        // Bonus for exact matches
        let exact_match_bonus = if text.to_lowercase() == search_text.to_lowercase() {
            0.8 // Higher bonus for exact matches
        } else if text.to_lowercase().starts_with(&search_text.to_lowercase())
            || text.to_lowercase().ends_with(&search_text.to_lowercase())
        {
            0.2 // Small bonus for prefix/suffix matches
        } else {
            0.0
        };

        // Bonus for word boundaries
        let word_boundary_bonus = matches
            .iter()
            .map(|m| {
                let at_start = m.start == 0
                    || !text
                        .chars()
                        .nth(m.start - 1)
                        .unwrap_or(' ')
                        .is_alphanumeric();
                let at_end = m.end >= text.len()
                    || !text.chars().nth(m.end).unwrap_or(' ').is_alphanumeric();
                if at_start && at_end {
                    0.2
                } else if at_start || at_end {
                    0.1
                } else {
                    0.0
                }
            })
            .sum::<f64>()
            / match_count;

        coverage + exact_match_bonus + word_boundary_bonus
    }

    /// Calculate bonus score based on credential properties
    fn calculate_bonus_score(credential: &CredentialRecord, query: &SearchQuery) -> f64 {
        let mut bonus = 0.0;

        // Favorite bonus
        if credential.favorite {
            bonus += 0.1;
        }

        // Recent access bonus (within 30 days gets bonus)
        let now = chrono::Utc::now().timestamp();
        let thirty_days = 30 * 24 * 60 * 60;
        if credential.accessed_at > now - thirty_days {
            bonus += 0.05;
        }

        // Tag match bonus
        let tag_matches = query
            .required_tags
            .iter()
            .chain(query.optional_tags.iter())
            .filter(|tag| credential.has_tag(tag))
            .count() as f64;

        if tag_matches > 0.0 {
            bonus += tag_matches * 0.02;
        }

        bonus
    }

    /// Find credentials with similar titles
    pub fn find_similar_titles(
        credentials: &HashMap<String, CredentialRecord>,
        title: &str,
        threshold: f64,
    ) -> Vec<SearchResult> {
        let mut results = Vec::new();

        for credential in credentials.values() {
            let similarity = Self::calculate_title_similarity(title, &credential.title);
            if similarity >= threshold {
                results.push(SearchResult {
                    credential: credential.clone(),
                    score: similarity,
                    matches: vec![],
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results
    }

    /// Calculate similarity between two titles using Levenshtein distance
    fn calculate_title_similarity(title1: &str, title2: &str) -> f64 {
        let title1_lower = title1.to_lowercase();
        let title2_lower = title2.to_lowercase();

        if title1_lower == title2_lower {
            return 1.0;
        }

        let distance = Self::levenshtein_distance(&title1_lower, &title2_lower);
        let max_len = title1_lower.len().max(title2_lower.len()) as f64;

        if max_len == 0.0 {
            return 1.0;
        }

        (max_len - distance as f64) / max_len
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(s1: &str, s2: &str) -> usize {
        let len1 = s1.chars().count();
        let len2 = s2.chars().count();

        if len1 == 0 {
            return len2;
        }
        if len2 == 0 {
            return len1;
        }

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }

        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        let s1_chars: Vec<char> = s1.chars().collect();
        let s2_chars: Vec<char> = s2.chars().collect();

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if s1_chars[i - 1] == s2_chars[j - 1] {
                    0
                } else {
                    1
                };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Get all unique tags from credentials
    pub fn extract_all_tags(credentials: &HashMap<String, CredentialRecord>) -> Vec<String> {
        let mut tags: HashSet<String> = HashSet::new();

        for credential in credentials.values() {
            for tag in &credential.tags {
                tags.insert(tag.clone());
            }
        }

        let mut sorted_tags: Vec<String> = tags.into_iter().collect();
        sorted_tags.sort();
        sorted_tags
    }

    /// Get all unique credential types
    pub fn extract_credential_types(
        credentials: &HashMap<String, CredentialRecord>,
    ) -> Vec<String> {
        let mut types: HashSet<String> = HashSet::new();

        for credential in credentials.values() {
            types.insert(credential.credential_type.clone());
        }

        let mut sorted_types: Vec<String> = types.into_iter().collect();
        sorted_types.sort();
        sorted_types
    }

    /// Get all unique folder paths
    pub fn extract_folder_paths(credentials: &HashMap<String, CredentialRecord>) -> Vec<String> {
        let mut paths: HashSet<String> = HashSet::new();

        for credential in credentials.values() {
            if let Some(folder_path) = &credential.folder_path {
                if !folder_path.is_empty() {
                    paths.insert(folder_path.clone());

                    // Also add parent folders
                    let mut path = folder_path.clone();
                    while let Some(pos) = path.rfind('/') {
                        path.truncate(pos);
                        if !path.is_empty() {
                            paths.insert(path.clone());
                        }
                    }
                }
            }
        }

        let mut sorted_paths: Vec<String> = paths.into_iter().collect();
        sorted_paths.sort();
        sorted_paths
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::CredentialField;

    fn create_test_credential(title: &str, cred_type: &str) -> CredentialRecord {
        let mut credential = CredentialRecord::new(title.to_string(), cred_type.to_string());
        credential.set_field("username", CredentialField::username("testuser"));
        credential.set_field("password", CredentialField::password("testpass"));
        credential
    }

    #[test]
    fn test_simple_text_search() {
        let mut credentials = HashMap::new();
        let credential1 = create_test_credential("Gmail Login", "login");
        let credential2 = create_test_credential("Bank Account", "login");

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);

        let query = SearchQuery::text("Gmail");
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].credential.title, "Gmail Login");
        assert!(!results[0].matches.is_empty());
    }

    #[test]
    fn test_tag_search() {
        let mut credentials = HashMap::new();
        let mut credential1 = create_test_credential("Work Email", "login");
        let mut credential2 = create_test_credential("Personal Email", "login");

        credential1.add_tag("work".to_string());
        credential2.add_tag("personal".to_string());

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);

        let query = SearchQuery::with_tags(vec!["work".to_string()]);
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].credential.title, "Work Email");
    }

    #[test]
    fn test_case_insensitive_search() {
        let mut credentials = HashMap::new();
        let credential = create_test_credential("Gmail Login", "login");
        credentials.insert(credential.id.clone(), credential);

        let query = SearchQuery::text("gmail").case_sensitive(false);
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_regex_search() {
        let mut credentials = HashMap::new();
        let credential1 = create_test_credential("Gmail Account", "login");
        let credential2 = create_test_credential("Yahoo Mail", "login");
        let credential3 = create_test_credential("Bank Login", "login");

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);
        credentials.insert(credential3.id.clone(), credential3);

        let query = SearchQuery::text(r"(Gmail|Yahoo)").with_regex(true);
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_field_value_search() {
        let mut credentials = HashMap::new();
        let mut credential = create_test_credential("Test Account", "login");
        credential.set_field("username", CredentialField::username("john.doe"));
        credentials.insert(credential.id.clone(), credential);

        let query = SearchQuery::text("john.doe");
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 1);
        assert!(results[0]
            .matches
            .iter()
            .any(|m| matches!(m.location, MatchLocation::FieldValue)));
    }

    #[test]
    fn test_sensitive_field_exclusion() {
        let mut credentials = HashMap::new();
        let mut credential = create_test_credential("Test Account", "login");
        credential.set_field("password", CredentialField::password("secretpass"));
        credentials.insert(credential.id.clone(), credential);

        // Search without including sensitive fields
        let query = SearchQuery::text("secretpass").include_sensitive(false);
        let results = CredentialSearchEngine::search(&credentials, &query);
        assert_eq!(results.len(), 0);

        // Search with sensitive fields included
        let query = SearchQuery::text("secretpass").include_sensitive(true);
        let results = CredentialSearchEngine::search(&credentials, &query);
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn test_favorites_filter() {
        let mut credentials = HashMap::new();
        let mut credential1 = create_test_credential("Favorite Account", "login");
        let credential2 = create_test_credential("Regular Account", "login");

        credential1.favorite = true;

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);

        let query = SearchQuery::default().favorites_only(true);
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].credential.title, "Favorite Account");
    }

    #[test]
    fn test_similarity_search() {
        let mut credentials = HashMap::new();
        let credential1 = create_test_credential("Gmail Account", "login");
        let credential2 = create_test_credential("Gmail Backup", "login");
        let credential3 = create_test_credential("Yahoo Mail", "login");

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);
        credentials.insert(credential3.id.clone(), credential3);

        let results = CredentialSearchEngine::find_similar_titles(&credentials, "Gmail", 0.3);
        assert!(results.len() >= 2); // Should find both Gmail entries

        // Results should be sorted by similarity
        assert!(results[0].score >= results[1].score);
    }

    #[test]
    fn test_extract_metadata() {
        let mut credentials = HashMap::new();

        let mut credential1 = create_test_credential("Work Email", "login");
        credential1.add_tag("work".to_string());
        credential1.add_tag("email".to_string());
        credential1.folder_path = Some("Work/Email".to_string());

        let mut credential2 = create_test_credential("Bank Account", "banking");
        credential2.add_tag("finance".to_string());
        credential2.folder_path = Some("Personal/Finance".to_string());

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);

        let tags = CredentialSearchEngine::extract_all_tags(&credentials);
        assert!(tags.contains(&"work".to_string()));
        assert!(tags.contains(&"email".to_string()));
        assert!(tags.contains(&"finance".to_string()));

        let types = CredentialSearchEngine::extract_credential_types(&credentials);
        assert!(types.contains(&"login".to_string()));
        assert!(types.contains(&"banking".to_string()));

        let paths = CredentialSearchEngine::extract_folder_paths(&credentials);
        assert!(paths.contains(&"Work".to_string()));
        assert!(paths.contains(&"Work/Email".to_string()));
        assert!(paths.contains(&"Personal".to_string()));
        assert!(paths.contains(&"Personal/Finance".to_string()));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(CredentialSearchEngine::levenshtein_distance("", ""), 0);
        assert_eq!(
            CredentialSearchEngine::levenshtein_distance("hello", "hello"),
            0
        );
        assert_eq!(
            CredentialSearchEngine::levenshtein_distance("hello", "helo"),
            1
        );
        assert_eq!(
            CredentialSearchEngine::levenshtein_distance("hello", "world"),
            4
        );
    }

    #[test]
    fn test_search_result_scoring() {
        let mut credentials = HashMap::new();

        // Exact title match should score highest
        let credential1 = create_test_credential("test", "login");

        // Partial title match should score lower
        let credential2 = create_test_credential("test account", "login");

        // Field match should score even lower
        let mut credential3 = create_test_credential("Account", "login");
        credential3.set_field("username", CredentialField::username("test"));

        credentials.insert(credential1.id.clone(), credential1);
        credentials.insert(credential2.id.clone(), credential2);
        credentials.insert(credential3.id.clone(), credential3);

        let query = SearchQuery::text("test");
        let results = CredentialSearchEngine::search(&credentials, &query);

        assert_eq!(results.len(), 3);

        // Results should be sorted by score
        assert!(results[0].score >= results[1].score);
        assert!(results[1].score >= results[2].score);

        // Exact title match should be first
        assert_eq!(results[0].credential.title, "test");
    }
}

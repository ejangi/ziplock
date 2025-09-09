//! Plugin system for ZipLock extensibility
//!
//! This module provides a flexible plugin architecture that allows extending
//! ZipLock's functionality without modifying core code. Plugins can provide
//! custom field types, credential templates, import/export formats, and
//! validation rules.

use crate::core::{CoreError, CoreResult};
use crate::models::{CommonTemplates, CredentialRecord, CredentialTemplate};
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Plugin capability flags
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    /// Can provide custom field types
    CustomFieldTypes,
    /// Can provide credential templates
    CredentialTemplates,
    /// Can provide import/export formats
    ImportExport,
    /// Can provide validation rules
    Validation,
    /// Can provide TOTP generators
    TotpGeneration,
    /// Can provide password generators
    PasswordGeneration,
    /// Can provide search filters
    SearchFilters,
    /// Can provide backup formats
    BackupFormats,
}

/// Plugin metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    /// Plugin unique identifier
    pub id: String,
    /// Plugin display name
    pub name: String,
    /// Plugin version
    pub version: String,
    /// Plugin description
    pub description: String,
    /// Plugin author
    pub author: String,
    /// Minimum ZipLock version required
    pub min_ziplock_version: String,
    /// Plugin capabilities
    pub capabilities: Vec<String>,
    /// Plugin configuration schema
    pub config_schema: Option<serde_json::Value>,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    /// Whether plugin is enabled
    pub enabled: bool,
    /// Plugin-specific configuration
    pub settings: HashMap<String, serde_json::Value>,
}

impl Default for PluginConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            settings: HashMap::new(),
        }
    }
}

/// Plugin execution context
#[derive(Debug)]
pub struct PluginContext {
    /// Plugin configuration
    pub config: PluginConfig,
    /// Shared data between plugins
    pub shared_data: HashMap<String, Box<dyn Any + Send + Sync>>,
}

impl PluginContext {
    pub fn new(config: PluginConfig) -> Self {
        Self {
            config,
            shared_data: HashMap::new(),
        }
    }
}

/// Custom field type definition
#[derive(Debug, Clone)]
pub struct CustomFieldType {
    /// Field type identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Field validation function
    pub validator: fn(&str) -> Result<(), String>,
    /// Field formatting function
    pub formatter: fn(&str) -> String,
    /// Whether field is sensitive by default
    pub default_sensitive: bool,
}

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Get plugin metadata
    fn metadata(&self) -> &PluginMetadata;

    /// Initialize the plugin
    fn initialize(&mut self, _context: &PluginContext) -> CoreResult<()> {
        Ok(())
    }

    /// Shutdown the plugin
    fn shutdown(&mut self) -> CoreResult<()> {
        Ok(())
    }

    /// Get plugin capabilities
    fn capabilities(&self) -> Vec<PluginCapability>;

    /// Handle plugin-specific operations
    fn handle_operation(&self, operation: &str, _data: &[u8]) -> CoreResult<Vec<u8>> {
        Err(CoreError::ValidationError {
            message: format!("Operation '{}' not supported by plugin", operation),
        })
    }

    /// Get the plugin as Any for downcasting
    fn as_any(&self) -> &dyn Any;
}

/// Field type provider trait
pub trait FieldTypeProvider: Plugin {
    /// Get custom field types provided by this plugin
    fn get_field_types(&self) -> Vec<CustomFieldType>;

    /// Validate a field value for a custom type
    fn validate_field(&self, field_type: &str, value: &str) -> CoreResult<()>;
}

/// Template provider trait
pub trait TemplateProvider: Plugin {
    /// Get credential templates provided by this plugin
    fn get_templates(&self) -> Vec<CredentialTemplate>;

    /// Create a credential from a template
    fn create_from_template(
        &self,
        template_id: &str,
        title: String,
    ) -> CoreResult<CredentialRecord>;
}

/// Import/Export provider trait
pub trait ImportExportProvider: Plugin {
    /// Get supported import formats
    fn get_import_formats(&self) -> Vec<String>;

    /// Get supported export formats
    fn get_export_formats(&self) -> Vec<String>;

    /// Import credentials from data
    fn import_credentials(&self, format: &str, data: &[u8]) -> CoreResult<Vec<CredentialRecord>>;

    /// Export credentials to data
    fn export_credentials(
        &self,
        format: &str,
        credentials: &[CredentialRecord],
    ) -> CoreResult<Vec<u8>>;
}

/// Validation provider trait
pub trait ValidationProvider: Plugin {
    /// Validate a credential
    fn validate_credential(&self, credential: &CredentialRecord) -> CoreResult<Vec<String>>;

    /// Get validation rules
    fn get_validation_rules(&self) -> Vec<ValidationRule>;
}

/// Validation rule definition
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub severity: ValidationSeverity,
    pub validator: fn(&CredentialRecord) -> bool,
}

/// Validation severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
}

/// Plugin registry for managing loaded plugins
pub struct PluginRegistry {
    plugins: RwLock<HashMap<String, Box<dyn Plugin>>>,
    configs: RwLock<HashMap<String, PluginConfig>>,
}

impl PluginRegistry {
    /// Create a new plugin registry
    pub fn new() -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            configs: RwLock::new(HashMap::new()),
        }
    }

    /// Register a plugin
    pub fn register_plugin(&self, plugin: Box<dyn Plugin>) -> CoreResult<()> {
        let plugin_id = plugin.metadata().id.clone();

        // Initialize plugin with default config
        let config = self.get_plugin_config(&plugin_id);
        let _context = PluginContext::new(config);

        {
            let mut plugins = self.plugins.write().map_err(|_| CoreError::InternalError {
                message: "Failed to acquire plugin registry lock".to_string(),
            })?;

            if plugins.contains_key(&plugin_id) {
                return Err(CoreError::ValidationError {
                    message: format!("Plugin '{}' is already registered", plugin_id),
                });
            }

            plugins.insert(plugin_id, plugin);
        }

        Ok(())
    }

    /// Unregister a plugin
    pub fn unregister_plugin(&self, plugin_id: &str) -> CoreResult<()> {
        let mut plugins = self.plugins.write().map_err(|_| CoreError::InternalError {
            message: "Failed to acquire plugin registry lock".to_string(),
        })?;

        if let Some(mut plugin) = plugins.remove(plugin_id) {
            plugin.shutdown()?;
        }

        Ok(())
    }

    /// Get a plugin by ID
    pub fn get_plugin(&self, plugin_id: &str) -> Option<PluginMetadata> {
        let plugins = self.plugins.read().unwrap();
        plugins.get(plugin_id).map(|p| p.metadata().clone())
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<PluginMetadata> {
        let plugins = match self.plugins.read() {
            Ok(plugins) => plugins,
            Err(_) => return Vec::new(),
        };

        plugins
            .values()
            .map(|plugin| plugin.metadata().clone())
            .collect()
    }

    /// Get plugins by capability
    pub fn get_plugins_by_capability(&self, capability: PluginCapability) -> Vec<PluginMetadata> {
        let plugins = match self.plugins.read() {
            Ok(plugins) => plugins,
            Err(_) => return Vec::new(),
        };

        plugins
            .values()
            .filter(|plugin| plugin.capabilities().contains(&capability))
            .map(|plugin| plugin.metadata().clone())
            .collect()
    }

    /// Set plugin configuration
    pub fn set_plugin_config(&self, plugin_id: &str, config: PluginConfig) -> CoreResult<()> {
        let mut configs = self.configs.write().map_err(|_| CoreError::InternalError {
            message: "Failed to acquire config lock".to_string(),
        })?;

        configs.insert(plugin_id.to_string(), config);
        Ok(())
    }

    /// Get plugin configuration
    pub fn get_plugin_config(&self, plugin_id: &str) -> PluginConfig {
        let configs = match self.configs.read() {
            Ok(configs) => configs,
            Err(_) => return PluginConfig::default(),
        };

        configs.get(plugin_id).cloned().unwrap_or_default()
    }

    /// Enable/disable a plugin
    pub fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> CoreResult<()> {
        let mut config = self.get_plugin_config(plugin_id);
        config.enabled = enabled;
        self.set_plugin_config(plugin_id, config)
    }

    /// Check if plugin is enabled
    pub fn is_plugin_enabled(&self, plugin_id: &str) -> bool {
        self.get_plugin_config(plugin_id).enabled
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Plugin manager for high-level plugin operations
pub struct PluginManager {
    registry: Arc<PluginRegistry>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            registry: Arc::new(PluginRegistry::new()),
        }
    }

    /// Get reference to plugin registry
    pub fn registry(&self) -> &PluginRegistry {
        &self.registry
    }

    /// Load all custom field types from plugins
    pub fn get_custom_field_types(&self) -> Vec<CustomFieldType> {
        let mut field_types = Vec::new();

        let plugins = self.registry.plugins.read().unwrap();
        for plugin in plugins.values() {
            if plugin
                .capabilities()
                .contains(&PluginCapability::CustomFieldTypes)
            {
                // Try to cast to BuiltinFieldTypeProvider
                if let Some(field_provider) = plugin
                    .as_any()
                    .downcast_ref::<builtin::BuiltinFieldTypeProvider>()
                {
                    field_types.extend(field_provider.get_field_types());
                }
            }
        }

        field_types
    }

    /// Load all templates from plugins
    pub fn get_plugin_templates(&self) -> Vec<CredentialTemplate> {
        let mut templates = Vec::new();

        let plugins = self.registry.plugins.read().unwrap();
        for plugin in plugins.values() {
            if plugin
                .capabilities()
                .contains(&PluginCapability::CredentialTemplates)
            {
                // Try to cast to BuiltinTemplateProvider
                if let Some(template_provider) = plugin
                    .as_any()
                    .downcast_ref::<builtin::BuiltinTemplateProvider>()
                {
                    templates.extend(template_provider.get_templates());
                }
            }
        }

        templates
    }

    /// Validate credential using all validation plugins
    pub fn validate_credential_with_plugins(&self, _credential: &CredentialRecord) -> Vec<String> {
        // Simplified implementation - return empty for now
        // TODO: Implement proper plugin system with concrete types

        Vec::new()
    }

    /// Get available import formats from plugins
    pub fn get_import_formats(&self) -> Vec<String> {
        // Simplified implementation - return empty for now
        // TODO: Implement proper plugin system with concrete types

        Vec::new()
    }

    /// Get available export formats from plugins
    pub fn get_export_formats(&self) -> Vec<String> {
        // Simplified implementation - return empty for now
        // TODO: Implement proper plugin system with concrete types

        Vec::new()
    }

    /// Import credentials using appropriate plugin
    pub fn import_with_plugin(
        &self,
        format: &str,
        _data: &[u8],
    ) -> CoreResult<Vec<CredentialRecord>> {
        // Simplified implementation - return error for now
        // TODO: Implement proper plugin system with concrete types

        Err(CoreError::ValidationError {
            message: format!("No plugin found for import format: {}", format),
        })
    }

    /// Export credentials using appropriate plugin
    pub fn export_with_plugin(
        &self,
        format: &str,
        _credentials: &[CredentialRecord],
    ) -> CoreResult<Vec<u8>> {
        // Simplified implementation - return error for now
        // TODO: Implement proper plugin system with concrete types

        Err(CoreError::ValidationError {
            message: format!("No plugin found for export format: {}", format),
        })
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait to add Any support to Plugin trait
/// TODO: Implement proper plugin system with concrete types
pub trait PluginAny: Plugin {
    fn as_any(&self) -> &dyn Any;
}

impl<T: Plugin + 'static> PluginAny for T {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Built-in plugins for core functionality
pub mod builtin {
    use super::*;

    /// Built-in template provider
    pub struct BuiltinTemplateProvider {
        metadata: PluginMetadata,
    }

    impl Default for BuiltinTemplateProvider {
        fn default() -> Self {
            Self::new()
        }
    }

    impl BuiltinTemplateProvider {
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: "ziplock.builtin.templates".to_string(),
                    name: "Built-in Templates".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    description: "Provides standard credential templates".to_string(),
                    author: "ZipLock Team".to_string(),
                    min_ziplock_version: "0.1.0".to_string(),
                    capabilities: vec!["CredentialTemplates".to_string()],
                    config_schema: None,
                },
            }
        }
    }

    impl Plugin for BuiltinTemplateProvider {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::CredentialTemplates]
        }

        fn initialize(&mut self, _context: &PluginContext) -> CoreResult<()> {
            Ok(())
        }

        fn handle_operation(&self, _operation: &str, _data: &[u8]) -> CoreResult<Vec<u8>> {
            Ok(vec![])
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl TemplateProvider for BuiltinTemplateProvider {
        fn get_templates(&self) -> Vec<CredentialTemplate> {
            vec![CommonTemplates::login(), CommonTemplates::secure_note()]
        }

        fn create_from_template(
            &self,
            template_id: &str,
            title: String,
        ) -> CoreResult<CredentialRecord> {
            match template_id {
                "login" => CommonTemplates::login()
                    .create_credential(title)
                    .map_err(|e| CoreError::ValidationError { message: e }),
                "secure_note" => CommonTemplates::secure_note()
                    .create_credential(title)
                    .map_err(|e| CoreError::ValidationError { message: e }),
                _ => Err(CoreError::ValidationError {
                    message: format!("Unknown template: {}", template_id),
                }),
            }
        }
    }

    /// Built-in field type provider
    pub struct BuiltinFieldTypeProvider {
        metadata: PluginMetadata,
    }

    impl Default for BuiltinFieldTypeProvider {
        fn default() -> Self {
            Self::new()
        }
    }

    impl BuiltinFieldTypeProvider {
        pub fn new() -> Self {
            Self {
                metadata: PluginMetadata {
                    id: "ziplock.builtin.fieldtypes".to_string(),
                    name: "Built-in Field Types".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    description: "Provides standard field types".to_string(),
                    author: "ZipLock Team".to_string(),
                    min_ziplock_version: "0.1.0".to_string(),
                    capabilities: vec!["CustomFieldTypes".to_string()],
                    config_schema: None,
                },
            }
        }
    }

    impl Plugin for BuiltinFieldTypeProvider {
        fn metadata(&self) -> &PluginMetadata {
            &self.metadata
        }

        fn capabilities(&self) -> Vec<PluginCapability> {
            vec![PluginCapability::CustomFieldTypes]
        }

        fn initialize(&mut self, _context: &PluginContext) -> CoreResult<()> {
            Ok(())
        }

        fn handle_operation(&self, _operation: &str, _data: &[u8]) -> CoreResult<Vec<u8>> {
            Ok(vec![])
        }

        fn as_any(&self) -> &dyn Any {
            self
        }
    }

    impl FieldTypeProvider for BuiltinFieldTypeProvider {
        fn get_field_types(&self) -> Vec<CustomFieldType> {
            vec![
                CustomFieldType {
                    id: "text".to_string(),
                    name: "Text".to_string(),
                    validator: |_| Ok(()),
                    formatter: |s| s.to_string(),
                    default_sensitive: false,
                },
                CustomFieldType {
                    id: "password".to_string(),
                    name: "Password".to_string(),
                    validator: |_| Ok(()),
                    formatter: |_| "••••••••".to_string(),
                    default_sensitive: true,
                },
                CustomFieldType {
                    id: "email".to_string(),
                    name: "Email".to_string(),
                    validator: |s| {
                        if s.contains('@') && s.contains('.') {
                            Ok(())
                        } else {
                            Err("Invalid email format".to_string())
                        }
                    },
                    formatter: |s| s.to_lowercase(),
                    default_sensitive: false,
                },
            ]
        }

        fn validate_field(&self, field_type: &str, value: &str) -> CoreResult<()> {
            for custom_type in self.get_field_types() {
                if custom_type.id == field_type {
                    return (custom_type.validator)(value)
                        .map_err(|e| CoreError::ValidationError { message: e });
                }
            }

            Err(CoreError::ValidationError {
                message: format!("Unknown field type: {}", field_type),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::builtin::*;
    use super::*;

    #[test]
    fn test_plugin_registry() {
        let registry = PluginRegistry::new();

        let plugin = Box::new(BuiltinTemplateProvider::new());
        let plugin_id = plugin.metadata().id.clone();

        assert!(registry.register_plugin(plugin).is_ok());
        assert!(registry.get_plugin(&plugin_id).is_some());

        let plugins = registry.list_plugins();
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, plugin_id);
    }

    #[test]
    fn test_plugin_capabilities() {
        let registry = PluginRegistry::new();

        let plugin = Box::new(BuiltinTemplateProvider::new());
        registry.register_plugin(plugin).unwrap();

        let template_plugins =
            registry.get_plugins_by_capability(PluginCapability::CredentialTemplates);
        assert_eq!(template_plugins.len(), 1);

        let import_plugins = registry.get_plugins_by_capability(PluginCapability::ImportExport);
        assert_eq!(import_plugins.len(), 0);
    }

    #[test]
    fn test_plugin_config() {
        let registry = PluginRegistry::new();
        let plugin_id = "test_plugin";

        let mut config = PluginConfig::default();
        config.enabled = false;

        registry.set_plugin_config(plugin_id, config).unwrap();

        let retrieved_config = registry.get_plugin_config(plugin_id);
        assert!(!retrieved_config.enabled);

        registry.set_plugin_enabled(plugin_id, true).unwrap();
        assert!(registry.is_plugin_enabled(plugin_id));
    }

    #[test]
    fn test_plugin_manager() {
        let manager = PluginManager::new();

        let template_plugin = Box::new(BuiltinTemplateProvider::new());
        let field_plugin = Box::new(BuiltinFieldTypeProvider::new());

        manager.registry().register_plugin(template_plugin).unwrap();
        manager.registry().register_plugin(field_plugin).unwrap();

        let templates = manager.get_plugin_templates();
        assert!(!templates.is_empty());

        let field_types = manager.get_custom_field_types();
        assert!(!field_types.is_empty());
    }

    #[test]
    fn test_builtin_template_provider() {
        let provider = BuiltinTemplateProvider::new();

        assert_eq!(
            provider.capabilities(),
            vec![PluginCapability::CredentialTemplates]
        );
        assert_eq!(provider.metadata().id, "ziplock.builtin.templates");

        let templates = provider.get_templates();
        assert!(!templates.is_empty());

        let login_cred = provider.create_from_template("login", "Test Login".to_string());
        assert!(login_cred.is_ok());

        let unknown_cred = provider.create_from_template("unknown", "Test".to_string());
        assert!(unknown_cred.is_err());
    }

    #[test]
    fn test_builtin_field_type_provider() {
        let provider = BuiltinFieldTypeProvider::new();

        assert_eq!(
            provider.capabilities(),
            vec![PluginCapability::CustomFieldTypes]
        );

        let field_types = provider.get_field_types();
        assert!(!field_types.is_empty());

        // Test email validation
        assert!(provider.validate_field("email", "test@example.com").is_ok());
        assert!(provider.validate_field("email", "invalid_email").is_err());

        // Test unknown field type
        assert!(provider.validate_field("unknown", "value").is_err());
    }

    #[test]
    fn test_validation_severity() {
        let severity = ValidationSeverity::Warning;
        assert_eq!(severity, ValidationSeverity::Warning);
        assert_ne!(severity, ValidationSeverity::Error);
    }

    #[test]
    fn test_plugin_metadata_serialization() {
        let metadata = PluginMetadata {
            id: "test.plugin".to_string(),
            name: "Test Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A test plugin".to_string(),
            author: "Test Author".to_string(),
            min_ziplock_version: "0.1.0".to_string(),
            capabilities: vec!["CustomFieldTypes".to_string()],
            config_schema: None,
        };

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: PluginMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata.id, deserialized.id);
        assert_eq!(metadata.name, deserialized.name);
    }
}

//! Model component implementation
//!
//! Displays the AI model name with optional custom mappings.

use super::base::{Component, ComponentFactory, ComponentOutput, RenderContext};
use crate::config::{BaseComponentConfig, Config, ModelComponentConfig};
use crate::utils::model_parser::parse_model_id;
use async_trait::async_trait;

/// Model component
pub struct ModelComponent {
    config: ModelComponentConfig,
}

impl ModelComponent {
    #[must_use]
    pub const fn new(config: ModelComponentConfig) -> Self {
        Self { config }
    }

    /// Get the model name to display
    fn get_model_name(&self, ctx: &RenderContext) -> Option<String> {
        let model = ctx.input.model.as_ref()?;

        // Priority 1: Check custom mappings first
        if let Some(id) = &model.id {
            if self.config.show_full_name {
                // Long name mode: check long_name_mapping
                if let Some(mapped) = self.config.long_name_mapping.get(id) {
                    return Some(mapped.clone());
                }
            } else {
                // Short name mode: check mapping
                if let Some(mapped) = self.config.mapping.get(id) {
                    return Some(mapped.clone());
                }
            }

            // Priority 2: Try intelligent parsing fallback
            if let Some(parsed) = parse_model_id(id) {
                return Some(if self.config.show_full_name {
                    parsed.long_name()
                } else {
                    parsed.short_name()
                });
            }

            // Priority 3: Final fallback - return original ID
            return Some(id.clone());
        }

        // No ID available, try display_name
        model.display_name.clone()
    }
}

#[async_trait]
impl Component for ModelComponent {
    fn name(&self) -> &'static str {
        "model"
    }

    fn is_enabled(&self, _ctx: &RenderContext) -> bool {
        self.config.base.enabled
    }

    async fn render(&self, ctx: &RenderContext) -> ComponentOutput {
        // Check if component is enabled
        if !self.is_enabled(ctx) {
            return ComponentOutput::hidden();
        }

        // Get model name
        let Some(text) = self.get_model_name(ctx) else {
            return ComponentOutput::hidden();
        };

        // Select icon
        let icon = self.select_icon(ctx);

        // Create output
        ComponentOutput::new(text)
            .with_icon(icon.unwrap_or_default())
            .with_icon_color(&self.config.base.icon_color)
            .with_text_color(&self.config.base.text_color)
    }

    fn base_config(&self, _ctx: &RenderContext) -> Option<&BaseComponentConfig> {
        Some(&self.config.base)
    }
}

/// Factory for creating Model components
pub struct ModelComponentFactory;

impl ComponentFactory for ModelComponentFactory {
    fn create(&self, config: &Config) -> Box<dyn Component> {
        Box::new(ModelComponent::new(config.components.model.clone()))
    }

    fn name(&self) -> &'static str {
        "model"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::TerminalCapabilities;
    use crate::core::{InputData, ModelInfo};
    use std::sync::Arc;

    #[allow(clippy::field_reassign_with_default)]
    fn build_model_config(
        configure: impl FnOnce(&mut ModelComponentConfig),
    ) -> ModelComponentConfig {
        let mut config = ModelComponentConfig::default();
        configure(&mut config);
        config
    }

    #[allow(clippy::field_reassign_with_default)]
    fn build_input(configure: impl FnOnce(&mut InputData)) -> InputData {
        let mut input = InputData::default();
        configure(&mut input);
        input
    }

    fn create_test_context_with_model(
        id: Option<String>,
        display_name: Option<String>,
    ) -> RenderContext {
        let input = build_input(|input| {
            input.model = Some(ModelInfo { id, display_name });
        });

        RenderContext {
            input: Arc::new(input),
            config: Arc::new(Config::default()),
            terminal: TerminalCapabilities::default(),
        }
    }

    // ==================== 短名称生成测试 ====================

    #[tokio::test]
    async fn test_short_name_with_params() {
        let component = ModelComponent::new(ModelComponentConfig::default());
        let ctx = create_test_context_with_model(
            Some("claude-sonnet-4-5-20250929[1m]".to_string()),
            None,
        );

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "S4.5[1m]");
    }

    #[tokio::test]
    async fn test_short_name_major_only() {
        let component = ModelComponent::new(ModelComponentConfig::default());
        let ctx = create_test_context_with_model(Some("claude-haiku-3-20240307".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "H3");
    }

    #[tokio::test]
    async fn test_short_name_opus() {
        let component = ModelComponent::new(ModelComponentConfig::default());
        let ctx =
            create_test_context_with_model(Some("claude-opus-4-1-20250805".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "O4.1");
    }

    // ==================== 长名称生成测试 ====================

    #[tokio::test]
    async fn test_long_name_with_params() {
        let config = build_model_config(|config| {
            config.show_full_name = true;
        });

        let component = ModelComponent::new(config);
        let ctx = create_test_context_with_model(
            Some("claude-sonnet-4-5-20250929[1m]".to_string()),
            None,
        );

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "Sonnet 4.5[1m]");
    }

    #[tokio::test]
    async fn test_long_name_major_only() {
        let config = build_model_config(|config| {
            config.show_full_name = true;
        });

        let component = ModelComponent::new(config);
        let ctx = create_test_context_with_model(Some("claude-haiku-3-20240307".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "Haiku 3");
    }

    #[tokio::test]
    async fn test_long_name_opus() {
        let config = build_model_config(|config| {
            config.show_full_name = true;
        });

        let component = ModelComponent::new(config);
        let ctx =
            create_test_context_with_model(Some("claude-opus-4-1-20250805".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "Opus 4.1");
    }

    // ==================== 配置映射优先级测试 ====================

    #[tokio::test]
    async fn test_custom_mapping_overrides_parsing() {
        let config = build_model_config(|config| {
            config.mapping.insert(
                "claude-sonnet-4-5-20250929[1m]".to_string(),
                "CustomS4.5".to_string(),
            );
        });

        let component = ModelComponent::new(config);
        let ctx = create_test_context_with_model(
            Some("claude-sonnet-4-5-20250929[1m]".to_string()),
            None,
        );

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "CustomS4.5");
    }

    #[tokio::test]
    async fn test_long_name_mapping_overrides_parsing() {
        let config = build_model_config(|config| {
            config.show_full_name = true;
            config.long_name_mapping.insert(
                "claude-opus-4-1-20250805".to_string(),
                "Custom Opus".to_string(),
            );
        });

        let component = ModelComponent::new(config);
        let ctx =
            create_test_context_with_model(Some("claude-opus-4-1-20250805".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "Custom Opus");
    }

    // ==================== 回退测试 ====================

    #[tokio::test]
    async fn test_fallback_to_original_id_on_parse_failure() {
        let component = ModelComponent::new(ModelComponentConfig::default());
        let ctx = create_test_context_with_model(Some("invalid-format".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(output.visible);
        // Should return original ID as final fallback
        assert_eq!(output.text, "invalid-format");
    }

    #[tokio::test]
    async fn test_fallback_to_display_name_when_no_id() {
        let component = ModelComponent::new(ModelComponentConfig::default());
        let ctx = create_test_context_with_model(None, Some("Some Model Name".to_string()));

        let output = component.render(&ctx).await;
        assert!(output.visible);
        assert_eq!(output.text, "Some Model Name");
    }

    // ==================== 边缘情况测试 ====================

    #[tokio::test]
    async fn test_model_disabled() {
        let config = build_model_config(|config| {
            config.base.enabled = false;
        });

        let component = ModelComponent::new(config);
        let ctx =
            create_test_context_with_model(Some("claude-sonnet-4-5-20250929".to_string()), None);

        let output = component.render(&ctx).await;
        assert!(!output.visible);
    }

    #[tokio::test]
    async fn test_no_model_info() {
        let component = ModelComponent::new(ModelComponentConfig::default());
        let input = build_input(|input| {
            input.model = None;
        });

        let ctx = RenderContext {
            input: Arc::new(input),
            config: Arc::new(Config::default()),
            terminal: TerminalCapabilities::default(),
        };

        let output = component.render(&ctx).await;
        assert!(!output.visible);
    }
}

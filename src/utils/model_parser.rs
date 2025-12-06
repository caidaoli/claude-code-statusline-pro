//! Model ID parsing utilities
//!
//! Provides common model ID parsing functionality used by multiple components.

/// Parse a model ID into structured components
///
/// Format: `{provider}-{series}-{major}-{minor}-{date}[{params}]`
/// Example: `claude-sonnet-4-5-20250929[1m]`
#[must_use]
pub fn parse_model_id(id: &str) -> Option<ParsedModelId> {
    // Extract params (e.g., "[1m]") if present
    let (base_id, params) = id.find('[').map_or((id, ""), |bracket_start| {
        (&id[..bracket_start], &id[bracket_start..])
    });

    // Split by '-' to get parts
    let parts: Vec<&str> = base_id.split('-').collect();

    // Expect at least: provider-series-major-...-date
    // Minimum 4 parts: provider-series-major-date
    if parts.len() < 4 {
        return None;
    }

    let provider = parts[0];
    let series = parts[1];

    // Determine version: could be major-minor-date or just major-date
    // Look for numeric patterns after series name
    let mut version_parts = Vec::new();
    let mut idx = 2;

    // Collect version numbers (major and optional minor)
    while idx < parts.len() {
        if parts[idx].parse::<u32>().is_ok() {
            // This looks like a version number or date
            // Date is always 8 digits (YYYYMMDD)
            if parts[idx].len() == 8 {
                // This is the date, stop here
                break;
            }
            version_parts.push(parts[idx]);
            idx += 1;
        } else {
            // Non-numeric part after series, invalid format
            return None;
        }
    }

    if version_parts.is_empty() {
        return None;
    }

    // Format version: "4-5" -> "4.5", "4" -> "4"
    let version = version_parts.join(".");

    Some(ParsedModelId {
        provider: provider.to_string(),
        series: series.to_string(),
        version,
        params: params.to_string(),
    })
}

/// Parsed model ID components
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedModelId {
    /// Provider name (e.g., "claude", future: "gemini", "gpt", etc.)
    pub provider: String,
    /// Series name (e.g., "sonnet", "opus", "haiku")
    pub series: String,
    /// Version string (e.g., "4.5", "3", "4.1")
    pub version: String,
    /// Parameters suffix (e.g., "[1m]" or "")
    pub params: String,
}

impl ParsedModelId {
    /// Generate short name (e.g., "S4.5[1m]")
    #[must_use]
    pub fn short_name(&self) -> String {
        let series_initial = self
            .series
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_default();

        format!("{}{}{}", series_initial, self.version, self.params)
    }

    /// Generate long name (e.g., "Sonnet 4.5[1m]")
    #[must_use]
    pub fn long_name(&self) -> String {
        let series_cap = capitalize(&self.series);

        format!("{} {}{}", series_cap, self.version, self.params)
    }

    /// Infer context window size from params
    ///
    /// Returns `Some(context_size)` if the params indicate a specific context window,
    /// otherwise returns `None` to use the default.
    #[must_use]
    pub fn infer_context_window(&self) -> Option<u64> {
        match self.params.as_str() {
            "[1m]" => Some(1_000_000),
            // Future extensions:
            // "[500k]" => Some(500_000),
            // "[2m]" => Some(2_000_000),
            _ => None,
        }
    }
}

/// Capitalize first letter of a string
#[must_use]
pub fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map_or_else(String::new, |first| {
        let mut result = first.to_uppercase().to_string();
        result.push_str(&chars.as_str().to_lowercase());
        result
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== 解析测试 ====================

    #[test]
    fn test_parse_model_id_with_minor_version_and_params() {
        let parsed = parse_model_id("claude-sonnet-4-5-20250929[1m]").unwrap();

        assert_eq!(parsed.provider, "claude");
        assert_eq!(parsed.series, "sonnet");
        assert_eq!(parsed.version, "4.5");
        assert_eq!(parsed.params, "[1m]");
    }

    #[test]
    fn test_parse_model_id_major_version_only() {
        let parsed = parse_model_id("claude-haiku-3-20240307").unwrap();

        assert_eq!(parsed.provider, "claude");
        assert_eq!(parsed.series, "haiku");
        assert_eq!(parsed.version, "3");
        assert_eq!(parsed.params, "");
    }

    #[test]
    fn test_parse_model_id_with_minor_no_params() {
        let parsed = parse_model_id("claude-opus-4-1-20250805").unwrap();

        assert_eq!(parsed.provider, "claude");
        assert_eq!(parsed.series, "opus");
        assert_eq!(parsed.version, "4.1");
        assert_eq!(parsed.params, "");
    }

    #[test]
    fn test_parse_model_id_invalid_format() {
        // Too few parts
        assert!(parse_model_id("claude-sonnet").is_none());

        // Non-numeric version
        assert!(parse_model_id("claude-sonnet-abc-20250929").is_none());
    }

    // ==================== 名称生成测试 ====================

    #[test]
    fn test_short_name_with_params() {
        let parsed = parse_model_id("claude-sonnet-4-5-20250929[1m]").unwrap();
        assert_eq!(parsed.short_name(), "S4.5[1m]");
    }

    #[test]
    fn test_short_name_without_params() {
        let parsed = parse_model_id("claude-opus-4-1-20250805").unwrap();
        assert_eq!(parsed.short_name(), "O4.1");
    }

    #[test]
    fn test_long_name_with_params() {
        let parsed = parse_model_id("claude-sonnet-4-5-20250929[1m]").unwrap();
        assert_eq!(parsed.long_name(), "Sonnet 4.5[1m]");
    }

    #[test]
    fn test_long_name_without_params() {
        let parsed = parse_model_id("claude-haiku-3-20240307").unwrap();
        assert_eq!(parsed.long_name(), "Haiku 3");
    }

    // ==================== 上下文推断测试 ====================

    #[test]
    fn test_infer_context_window_1m() {
        let parsed = parse_model_id("claude-sonnet-4-5-20250929[1m]").unwrap();
        assert_eq!(parsed.infer_context_window(), Some(1_000_000));
    }

    #[test]
    fn test_infer_context_window_no_params() {
        let parsed = parse_model_id("claude-opus-4-1-20250805").unwrap();
        assert_eq!(parsed.infer_context_window(), None);
    }

    #[test]
    fn test_infer_context_window_unknown_params() {
        let parsed = parse_model_id("claude-sonnet-4-5-20250929[unknown]").unwrap();
        assert_eq!(parsed.infer_context_window(), None);
    }

    // ==================== 辅助函数测试 ====================

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("sonnet"), "Sonnet");
        assert_eq!(capitalize("OPUS"), "Opus");
        assert_eq!(capitalize("h"), "H");
        assert_eq!(capitalize(""), "");
    }
}

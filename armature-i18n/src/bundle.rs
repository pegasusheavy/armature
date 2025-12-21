//! Fluent Message Bundle
//!
//! Provides Mozilla Fluent integration for advanced message formatting.
//!
//! Fluent is a localization system designed for natural-sounding translations
//! with features like:
//! - Automatic plural rules
//! - Gender agreement
//! - Contextual formatting
//! - Asymmetric localization

use crate::{I18nError, Locale, Result};
use std::collections::HashMap;

/// Fluent bundle wrapper.
///
/// When the `fluent` feature is enabled, this provides a wrapper around
/// the `fluent-bundle` crate. Otherwise, it provides a simple fallback.
pub struct FluentBundle {
    #[cfg(feature = "fluent")]
    bundle: fluent_bundle::FluentBundle<fluent_bundle::FluentResource>,
    #[cfg(not(feature = "fluent"))]
    messages: HashMap<String, String>,
    locale: Locale,
}

impl std::fmt::Debug for FluentBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FluentBundle")
            .field("locale", &self.locale)
            .finish_non_exhaustive()
    }
}

impl FluentBundle {
    /// Create a new Fluent bundle.
    #[cfg(feature = "fluent")]
    pub fn new(locale: &Locale) -> Result<Self> {
        use unic_langid::LanguageIdentifier;

        let lang_id: LanguageIdentifier = locale
            .tag()
            .parse()
            .map_err(|e| I18nError::InvalidLocale(format!("{}: {}", locale.tag(), e)))?;

        let bundle = fluent_bundle::FluentBundle::new(vec![lang_id]);

        Ok(Self {
            bundle,
            locale: locale.clone(),
        })
    }

    /// Create a new bundle (fallback without fluent feature).
    #[cfg(not(feature = "fluent"))]
    pub fn new(locale: &Locale) -> Result<Self> {
        Ok(Self {
            messages: HashMap::new(),
            locale: locale.clone(),
        })
    }

    /// Add a Fluent resource (.ftl content).
    #[cfg(feature = "fluent")]
    pub fn add_resource(&mut self, source: &str) -> Result<()> {
        let resource =
            fluent_bundle::FluentResource::try_new(source.to_string()).map_err(|(_, errors)| {
                I18nError::FluentError(
                    errors
                        .into_iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join(", "),
                )
            })?;

        self.bundle.add_resource(resource).map_err(|errors| {
            I18nError::FluentError(
                errors
                    .into_iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })?;

        Ok(())
    }

    /// Add resource (fallback).
    #[cfg(not(feature = "fluent"))]
    pub fn add_resource(&mut self, source: &str) -> Result<()> {
        // Simple parser for basic Fluent syntax
        for line in source.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();
                self.messages.insert(key.to_string(), value.to_string());
            }
        }
        Ok(())
    }

    /// Format a message.
    #[cfg(feature = "fluent")]
    pub fn format(&self, key: &str, args: Option<&HashMap<String, FluentValue>>) -> Result<String> {
        use fluent_bundle::FluentArgs;

        let msg = self
            .bundle
            .get_message(key)
            .ok_or_else(|| I18nError::MessageNotFound {
                key: key.to_string(),
                locale: self.locale.tag(),
            })?;

        let pattern = msg.value().ok_or_else(|| I18nError::MessageNotFound {
            key: key.to_string(),
            locale: self.locale.tag(),
        })?;

        let mut errors = vec![];

        let fluent_args = args.map(|a| {
            let mut fa = FluentArgs::new();
            for (k, v) in a {
                fa.set(k.as_str(), v.to_fluent_value());
            }
            fa
        });

        let result = self
            .bundle
            .format_pattern(pattern, fluent_args.as_ref(), &mut errors);

        if !errors.is_empty() {
            return Err(I18nError::FluentError(
                errors
                    .into_iter()
                    .map(|e| format!("{:?}", e))
                    .collect::<Vec<_>>()
                    .join(", "),
            ));
        }

        Ok(result.to_string())
    }

    /// Format a message (fallback).
    #[cfg(not(feature = "fluent"))]
    pub fn format(&self, key: &str, args: Option<&HashMap<String, FluentValue>>) -> Result<String> {
        let msg = self
            .messages
            .get(key)
            .ok_or_else(|| I18nError::MessageNotFound {
                key: key.to_string(),
                locale: self.locale.tag(),
            })?;

        let mut result = msg.clone();

        if let Some(args) = args {
            for (k, v) in args {
                let placeholder = format!("{{ ${} }}", k);
                let placeholder2 = format!("{{${}}}", k);
                result = result.replace(&placeholder, &v.to_string());
                result = result.replace(&placeholder2, &v.to_string());
            }
        }

        Ok(result)
    }

    /// Check if a message exists.
    #[cfg(feature = "fluent")]
    pub fn has_message(&self, key: &str) -> bool {
        self.bundle.has_message(key)
    }

    /// Check if a message exists (fallback).
    #[cfg(not(feature = "fluent"))]
    pub fn has_message(&self, key: &str) -> bool {
        self.messages.contains_key(key)
    }

    /// Get the locale.
    pub fn locale(&self) -> &Locale {
        &self.locale
    }
}

/// Value type for Fluent arguments.
#[derive(Debug, Clone)]
pub enum FluentValue {
    String(String),
    Number(f64),
}

impl FluentValue {
    #[cfg(feature = "fluent")]
    fn to_fluent_value(&self) -> fluent_bundle::FluentValue<'static> {
        match self {
            FluentValue::String(s) => fluent_bundle::FluentValue::from(s.clone()),
            FluentValue::Number(n) => fluent_bundle::FluentValue::from(*n),
        }
    }
}

impl std::fmt::Display for FluentValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FluentValue::String(s) => write!(f, "{}", s),
            FluentValue::Number(n) => write!(f, "{}", n),
        }
    }
}

impl From<String> for FluentValue {
    fn from(s: String) -> Self {
        FluentValue::String(s)
    }
}

impl From<&str> for FluentValue {
    fn from(s: &str) -> Self {
        FluentValue::String(s.to_string())
    }
}

impl From<f64> for FluentValue {
    fn from(n: f64) -> Self {
        FluentValue::Number(n)
    }
}

impl From<i32> for FluentValue {
    fn from(n: i32) -> Self {
        FluentValue::Number(n as f64)
    }
}

impl From<i64> for FluentValue {
    fn from(n: i64) -> Self {
        FluentValue::Number(n as f64)
    }
}

impl From<usize> for FluentValue {
    fn from(n: usize) -> Self {
        FluentValue::Number(n as f64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fluent_bundle_simple() {
        let mut bundle = FluentBundle::new(&Locale::en()).unwrap();
        bundle.add_resource("hello = Hello, World!").unwrap();

        let result = bundle.format("hello", None).unwrap();
        assert_eq!(result, "Hello, World!");
    }

    #[test]
    fn test_fluent_bundle_with_args() {
        let mut bundle = FluentBundle::new(&Locale::en()).unwrap();
        bundle.add_resource("greeting = Hello, { $name }!").unwrap();

        let mut args = HashMap::new();
        args.insert("name".to_string(), FluentValue::from("Alice"));

        let result = bundle.format("greeting", Some(&args)).unwrap();
        assert!(result.contains("Alice"));
    }

    #[test]
    fn test_fluent_value_conversions() {
        let _s: FluentValue = "test".into();
        let _n: FluentValue = 42.into();
        let _f: FluentValue = 3.5.into();
    }
}

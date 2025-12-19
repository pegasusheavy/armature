//! Push notification message types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Notification priority.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    /// Normal priority.
    #[default]
    Normal,
    /// High priority (may wake device).
    High,
}

/// Web Push urgency level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Urgency {
    /// Very low urgency (device may delay significantly).
    VeryLow,
    /// Low urgency.
    Low,
    /// Normal urgency.
    #[default]
    Normal,
    /// High urgency (deliver immediately).
    High,
}

/// Push notification content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    /// Notification title.
    pub title: String,
    /// Notification body.
    pub body: String,
    /// Icon URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Image URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    /// Badge count (iOS).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub badge: Option<u32>,
    /// Sound to play.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<String>,
    /// Click action URL or intent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click_action: Option<String>,
    /// Tag for notification grouping.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Custom data payload.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub data: HashMap<String, String>,
    /// Notification priority.
    #[serde(default)]
    pub priority: Priority,
    /// Web Push urgency.
    #[serde(default)]
    pub urgency: Urgency,
    /// Time to live in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl: Option<u32>,
    /// Topic for iOS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub topic: Option<String>,
    /// Collapse key for Android.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collapse_key: Option<String>,
    /// Silent notification (data only).
    #[serde(default)]
    pub silent: bool,
    /// Mutable content (iOS).
    #[serde(default)]
    pub mutable_content: bool,
    /// Content available (iOS background).
    #[serde(default)]
    pub content_available: bool,
    /// Action buttons.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<NotificationAction>,
}

impl Notification {
    /// Create a new notification.
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            icon: None,
            image: None,
            badge: None,
            sound: None,
            click_action: None,
            tag: None,
            data: HashMap::new(),
            priority: Priority::Normal,
            urgency: Urgency::Normal,
            ttl: None,
            topic: None,
            collapse_key: None,
            silent: false,
            mutable_content: false,
            content_available: false,
            actions: Vec::new(),
        }
    }

    /// Create a builder.
    pub fn builder() -> NotificationBuilder {
        NotificationBuilder::new()
    }

    /// Create a silent/data-only notification.
    pub fn data_only() -> Self {
        Self {
            title: String::new(),
            body: String::new(),
            silent: true,
            content_available: true,
            ..Default::default()
        }
    }

    /// Set the icon URL.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set the image URL.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.image = Some(image.into());
        self
    }

    /// Set the badge count.
    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    /// Set the sound.
    pub fn sound(mut self, sound: impl Into<String>) -> Self {
        self.sound = Some(sound.into());
        self
    }

    /// Set the click action.
    pub fn click_action(mut self, action: impl Into<String>) -> Self {
        self.click_action = Some(action.into());
        self
    }

    /// Set the tag for grouping.
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tag = Some(tag.into());
        self
    }

    /// Add custom data.
    pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.data.insert(key.into(), value.into());
        self
    }

    /// Set priority.
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set high priority.
    pub fn high_priority(mut self) -> Self {
        self.priority = Priority::High;
        self.urgency = Urgency::High;
        self
    }

    /// Set urgency (Web Push).
    pub fn urgency(mut self, urgency: Urgency) -> Self {
        self.urgency = urgency;
        self
    }

    /// Set time to live.
    pub fn ttl(mut self, seconds: u32) -> Self {
        self.ttl = Some(seconds);
        self
    }

    /// Set topic (iOS).
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.topic = Some(topic.into());
        self
    }

    /// Set collapse key (Android).
    pub fn collapse_key(mut self, key: impl Into<String>) -> Self {
        self.collapse_key = Some(key.into());
        self
    }

    /// Make this a silent notification.
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self.content_available = true;
        self
    }

    /// Enable mutable content (iOS).
    pub fn mutable_content(mut self) -> Self {
        self.mutable_content = true;
        self
    }

    /// Add an action button.
    pub fn action(mut self, action: NotificationAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Get the payload size.
    pub fn payload_size(&self) -> usize {
        serde_json::to_string(self).map(|s| s.len()).unwrap_or(0)
    }
}

impl Default for Notification {
    fn default() -> Self {
        Self::new("", "")
    }
}

/// Notification action button.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationAction {
    /// Action identifier.
    pub action: String,
    /// Button title.
    pub title: String,
    /// Icon URL (Web Push).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
}

impl NotificationAction {
    /// Create a new action.
    pub fn new(action: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            action: action.into(),
            title: title.into(),
            icon: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Builder for notifications.
#[derive(Default)]
pub struct NotificationBuilder {
    notification: Notification,
}

impl NotificationBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.notification.title = title.into();
        self
    }

    /// Set the body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.notification.body = body.into();
        self
    }

    /// Set the icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.notification.icon = Some(icon.into());
        self
    }

    /// Set the image.
    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.notification.image = Some(image.into());
        self
    }

    /// Set the badge.
    pub fn badge(mut self, badge: u32) -> Self {
        self.notification.badge = Some(badge);
        self
    }

    /// Add data.
    pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.notification.data.insert(key.into(), value.into());
        self
    }

    /// Set priority.
    pub fn priority(mut self, priority: Priority) -> Self {
        self.notification.priority = priority;
        self
    }

    /// Build the notification.
    pub fn build(self) -> Notification {
        self.notification
    }
}

//! Device subscription types.

use serde::{Deserialize, Serialize};

/// Platform type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// Web browser.
    Web,
    /// Android device.
    Android,
    /// iOS device.
    Ios,
}

/// Device token for push notifications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceToken {
    /// The device token or registration ID.
    pub token: String,
    /// Platform type.
    pub platform: Platform,
    /// User ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Device ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_id: Option<String>,
    /// App version (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_version: Option<String>,
    /// Last active timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active: Option<i64>,
    /// Created timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<i64>,
}

impl DeviceToken {
    /// Create a new device token.
    pub fn new(token: impl Into<String>, platform: Platform) -> Self {
        Self {
            token: token.into(),
            platform,
            user_id: None,
            device_id: None,
            app_version: None,
            last_active: None,
            created_at: None,
        }
    }

    /// Create an Android device token.
    pub fn android(token: impl Into<String>) -> Self {
        Self::new(token, Platform::Android)
    }

    /// Create an iOS device token.
    pub fn ios(token: impl Into<String>) -> Self {
        Self::new(token, Platform::Ios)
    }

    /// Create a web push subscription.
    pub fn web(token: impl Into<String>) -> Self {
        Self::new(token, Platform::Web)
    }

    /// Set the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set the device ID.
    pub fn device_id(mut self, device_id: impl Into<String>) -> Self {
        self.device_id = Some(device_id.into());
        self
    }
}

/// Web Push subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    /// Endpoint URL.
    pub endpoint: String,
    /// Expiration time (Unix timestamp).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expiration_time: Option<i64>,
    /// Subscription keys.
    pub keys: SubscriptionKeys,
    /// User ID (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
}

/// Web Push subscription keys.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionKeys {
    /// p256dh key.
    pub p256dh: String,
    /// Auth secret.
    pub auth: String,
}

impl Subscription {
    /// Create a new subscription.
    pub fn new(
        endpoint: impl Into<String>,
        p256dh: impl Into<String>,
        auth: impl Into<String>,
    ) -> Self {
        Self {
            endpoint: endpoint.into(),
            expiration_time: None,
            keys: SubscriptionKeys {
                p256dh: p256dh.into(),
                auth: auth.into(),
            },
            user_id: None,
        }
    }

    /// Set the user ID.
    pub fn user_id(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Check if the subscription is expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expiration) = self.expiration_time {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;
            expiration < now
        } else {
            false
        }
    }
}

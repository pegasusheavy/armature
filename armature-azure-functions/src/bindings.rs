//! Azure Functions bindings support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Input binding trait.
pub trait InputBinding: Send + Sync {
    /// Get the binding name.
    fn name(&self) -> &str;

    /// Get the binding type.
    fn binding_type(&self) -> &str;
}

/// Output binding trait.
pub trait OutputBinding: Send + Sync {
    /// Get the binding name.
    fn name(&self) -> &str;

    /// Get the binding type.
    fn binding_type(&self) -> &str;
}

/// Blob storage input binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobInput {
    /// Binding name.
    pub name: String,
    /// Blob path.
    pub path: String,
    /// Connection string name.
    pub connection: String,
    /// Blob content.
    #[serde(default)]
    pub content: Vec<u8>,
}

impl InputBinding for BlobInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "blob"
    }
}

/// Blob storage output binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlobOutput {
    /// Binding name.
    pub name: String,
    /// Blob path.
    pub path: String,
    /// Connection string name.
    pub connection: String,
    /// Blob content.
    pub content: Vec<u8>,
}

impl OutputBinding for BlobOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "blob"
    }
}

/// Queue storage input binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueInput {
    /// Binding name.
    pub name: String,
    /// Queue name.
    pub queue_name: String,
    /// Connection string name.
    pub connection: String,
    /// Message content.
    #[serde(default)]
    pub message: String,
}

impl InputBinding for QueueInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "queue"
    }
}

/// Queue storage output binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueOutput {
    /// Binding name.
    pub name: String,
    /// Queue name.
    pub queue_name: String,
    /// Connection string name.
    pub connection: String,
    /// Message content.
    pub message: String,
}

impl OutputBinding for QueueOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "queue"
    }
}

/// Cosmos DB input binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDbInput {
    /// Binding name.
    pub name: String,
    /// Database name.
    pub database_name: String,
    /// Collection name.
    pub collection_name: String,
    /// Connection string name.
    pub connection: String,
    /// SQL query.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sql_query: Option<String>,
    /// Document ID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Partition key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition_key: Option<String>,
    /// Retrieved documents.
    #[serde(default)]
    pub documents: Vec<serde_json::Value>,
}

impl InputBinding for CosmosDbInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "cosmosDB"
    }
}

/// Cosmos DB output binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosmosDbOutput {
    /// Binding name.
    pub name: String,
    /// Database name.
    pub database_name: String,
    /// Collection name.
    pub collection_name: String,
    /// Connection string name.
    pub connection: String,
    /// Documents to write.
    pub documents: Vec<serde_json::Value>,
}

impl OutputBinding for CosmosDbOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "cosmosDB"
    }
}

/// Service Bus queue input binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBusQueueInput {
    /// Binding name.
    pub name: String,
    /// Queue name.
    pub queue_name: String,
    /// Connection string name.
    pub connection: String,
    /// Message content.
    #[serde(default)]
    pub message: String,
    /// Message properties.
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

impl InputBinding for ServiceBusQueueInput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "serviceBusTrigger"
    }
}

/// Service Bus queue output binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceBusQueueOutput {
    /// Binding name.
    pub name: String,
    /// Queue name.
    pub queue_name: String,
    /// Connection string name.
    pub connection: String,
    /// Message content.
    pub message: String,
    /// Message properties.
    #[serde(default)]
    pub properties: HashMap<String, String>,
}

impl OutputBinding for ServiceBusQueueOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "serviceBus"
    }
}

/// Event Grid output binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventGridOutput {
    /// Binding name.
    pub name: String,
    /// Topic endpoint.
    pub topic_endpoint: String,
    /// Topic key setting name.
    pub topic_key_setting: String,
    /// Events to publish.
    pub events: Vec<EventGridEvent>,
}

/// Event Grid event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventGridEvent {
    /// Event ID.
    pub id: String,
    /// Event type.
    pub event_type: String,
    /// Subject.
    pub subject: String,
    /// Event time.
    pub event_time: String,
    /// Data.
    pub data: serde_json::Value,
    /// Data version.
    pub data_version: String,
}

impl OutputBinding for EventGridOutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "eventGrid"
    }
}

/// SignalR output binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalROutput {
    /// Binding name.
    pub name: String,
    /// Hub name.
    pub hub_name: String,
    /// Connection string name.
    pub connection: String,
    /// Target method.
    pub target: String,
    /// Arguments.
    pub arguments: Vec<serde_json::Value>,
    /// User ID (optional, for sending to specific user).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// Group name (optional, for sending to group).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_name: Option<String>,
}

impl OutputBinding for SignalROutput {
    fn name(&self) -> &str {
        &self.name
    }

    fn binding_type(&self) -> &str {
        "signalR"
    }
}

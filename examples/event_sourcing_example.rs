//! Event Sourcing Example
//!
//! Demonstrates event-sourced aggregates with event store and snapshots.

use armature_events::DomainEvent;
use armature_eventsourcing::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Define aggregate state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct BankAccountState {
    balance: f64,
    owner: String,
    is_active: bool,
}

// Define aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
struct BankAccountAggregate {
    id: String,
    version: u64,
    state: BankAccountState,
    #[serde(skip)]
    pending_events: Vec<DomainEvent>,
}

impl BankAccountAggregate {
    fn new(id: String) -> Self {
        Self {
            id,
            version: 0,
            state: BankAccountState::default(),
            pending_events: Vec::new(),
        }
    }

    // Business methods
    fn deposit(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        if !self.state.is_active {
            return Err("Account is not active".to_string());
        }

        let event = DomainEvent::new(
            "money_deposited",
            &self.id,
            "BankAccount",
            serde_json::json!({ "amount": amount }),
        );

        // Apply event to update state
        self.apply_event(&event).unwrap();
        self.pending_events.push(event);

        Ok(())
    }

    fn withdraw(&mut self, amount: f64) -> Result<(), String> {
        if amount <= 0.0 {
            return Err("Amount must be positive".to_string());
        }

        if !self.state.is_active {
            return Err("Account is not active".to_string());
        }

        if self.state.balance < amount {
            return Err("Insufficient funds".to_string());
        }

        let event = DomainEvent::new(
            "money_withdrawn",
            &self.id,
            "BankAccount",
            serde_json::json!({ "amount": amount }),
        );

        self.apply_event(&event).unwrap();
        self.pending_events.push(event);

        Ok(())
    }

    fn deactivate(&mut self) -> Result<(), String> {
        if !self.state.is_active {
            return Err("Account already deactivated".to_string());
        }

        let event = DomainEvent::new(
            "account_deactivated",
            &self.id,
            "BankAccount",
            serde_json::json!({}),
        );

        self.apply_event(&event).unwrap();
        self.pending_events.push(event);

        Ok(())
    }
}

#[async_trait]
impl Aggregate for BankAccountAggregate {
    fn aggregate_id(&self) -> &str {
        &self.id
    }

    fn aggregate_type() -> &'static str {
        "BankAccount"
    }

    fn version(&self) -> u64 {
        self.version
    }

    fn apply_event(&mut self, event: &DomainEvent) -> Result<(), AggregateError> {
        match event.metadata.name.as_str() {
            "account_created" => {
                let owner = event.payload["owner"].as_str().unwrap();
                self.state.owner = owner.to_string();
                self.state.balance = 0.0;
                self.state.is_active = true;
                self.version += 1;
            }
            "money_deposited" => {
                let amount = event.payload["amount"].as_f64().unwrap();
                self.state.balance += amount;
                self.version += 1;
            }
            "money_withdrawn" => {
                let amount = event.payload["amount"].as_f64().unwrap();
                self.state.balance -= amount;
                self.version += 1;
            }
            "account_deactivated" => {
                self.state.is_active = false;
                self.version += 1;
            }
            _ => {}
        }
        Ok(())
    }

    fn uncommitted_events(&self) -> &[DomainEvent] {
        &self.pending_events
    }

    fn mark_events_committed(&mut self) {
        self.pending_events.clear();
    }

    fn new_instance(id: String) -> Self {
        Self::new(id)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Event Sourcing Example ===\n");

    // 1. Create event store
    println!("1. Creating Event Store:");
    let store = Arc::new(InMemoryEventStore::new());
    println!("   ✅ In-memory event store created\n");

    // 2. Create repository
    println!("2. Creating Repository:");
    let repo = AggregateRepository::<BankAccountAggregate, _>::new(store.clone());
    println!("   ✅ Aggregate repository created\n");

    // 3. Create new bank account
    println!("3. Creating Bank Account:");
    let account_id = "account-123";
    let mut account = BankAccountAggregate::new(account_id.to_string());

    // Add creation event
    let creation_event = DomainEvent::new(
        "account_created",
        account_id,
        "BankAccount",
        serde_json::json!({ "owner": "Alice" }),
    );
    account.apply_event(&creation_event)?;
    account.pending_events.push(creation_event);

    println!("   Account ID: {}", account_id);
    println!("   Owner: {}", account.state.owner);
    println!("   Balance: ${:.2}", account.state.balance);
    println!("   Version: {}", account.version());
    println!();

    // 4. Save aggregate
    println!("4. Saving Aggregate:");
    repo.save(&mut account).await?;
    println!("   ✅ Events persisted to event store\n");

    // 5. Make deposits
    println!("5. Making Deposits:");
    account.deposit(100.0)?;
    println!("   Deposited: $100.00");
    account.deposit(50.0)?;
    println!("   Deposited: $50.00");
    println!("   New balance: ${:.2}", account.state.balance);
    println!("   Version: {}", account.version());
    println!();

    // Save again
    repo.save(&mut account).await?;
    println!("   ✅ Events saved\n");

    // 6. Make withdrawal
    println!("6. Making Withdrawal:");
    account.withdraw(30.0)?;
    println!("   Withdrew: $30.00");
    println!("   New balance: ${:.2}", account.state.balance);
    repo.save(&mut account).await?;
    println!();

    // 7. Load aggregate from event store
    println!("7. Loading Aggregate from Event Store:");
    let loaded_account = repo.load(account_id).await?;
    println!("   Account ID: {}", loaded_account.aggregate_id());
    println!("   Owner: {}", loaded_account.state.owner);
    println!("   Balance: ${:.2}", loaded_account.state.balance);
    println!("   Active: {}", loaded_account.state.is_active);
    println!("   Version: {}", loaded_account.version());
    println!();

    // 8. View all events
    println!("8. Event History:");
    let events = store.load_events(account_id, None).await?;
    println!("   Total events: {}", events.len());
    for (i, event) in events.iter().enumerate() {
        println!(
            "   {}. {} (version {})",
            i + 1,
            event.metadata.name,
            event.version
        );
        println!("      Payload: {}", event.payload);
    }
    println!();

    // 9. Deactivate account
    println!("9. Deactivating Account:");
    let mut account_mut = loaded_account;
    account_mut.deactivate()?;
    repo.save(&mut account_mut).await?;
    println!("   ✅ Account deactivated\n");

    // 10. Try to deposit (should fail)
    println!("10. Attempting Deposit on Deactivated Account:");
    match account_mut.deposit(25.0) {
        Ok(_) => println!("   ❌ Unexpected: Deposit succeeded"),
        Err(e) => println!("   ✅ Expected error: {}", e),
    }
    println!();

    // 11. Demonstrate snapshots
    println!("11. Repository with Snapshots:");
    let _snapshot_repo = AggregateRepository::<BankAccountAggregate, _>::with_snapshots(
        store.clone(),
        3, // Snapshot every 3 events
    );
    println!("   ✅ Repository configured with snapshotting (every 3 events)\n");

    println!("=== Event Sourcing Example Complete ===\n");
    println!("💡 Key Features Demonstrated:");
    println!("   ✅ Event-sourced aggregates");
    println!("   ✅ Domain events");
    println!("   ✅ Event store (in-memory)");
    println!("   ✅ Aggregate repository");
    println!("   ✅ Business logic in aggregates");
    println!("   ✅ Event replay (load from history)");
    println!("   ✅ Optimistic concurrency");
    println!("   ✅ Snapshot configuration");
    println!();

    Ok(())
}

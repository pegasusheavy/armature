//! Contract Testing Example
//!
//! Demonstrates Pact-style consumer-driven contract testing.

use armature_testing::contract::*;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Contract Testing Example ===\n");

    // 1. Create a consumer contract
    println!("1. Creating Consumer Contract:");
    println!("   Consumer: Frontend");
    println!("   Provider: UserAPI\n");

    let mut builder = ContractBuilder::new("Frontend", "UserAPI");

    // Interaction 1: Get user by ID
    let get_user_request = ContractRequest::new(ContractMethod::Get, "/api/users/1")
        .with_header("Accept", "application/json");

    let get_user_response = ContractResponse::new(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com"
        }));

    builder.add_interaction(
        ContractInteraction::new("get user by ID", get_user_request, get_user_response)
            .with_provider_state("user with ID 1 exists"),
    );

    // Interaction 2: Create user
    let create_user_request = ContractRequest::new(ContractMethod::Post, "/api/users")
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "name": "Bob",
            "email": "bob@example.com"
        }));

    let create_user_response = ContractResponse::new(201)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "id": 2,
            "name": "Bob",
            "email": "bob@example.com"
        }));

    builder.add_interaction(ContractInteraction::new(
        "create new user",
        create_user_request,
        create_user_response,
    ));

    // Interaction 3: Update user
    let update_user_request = ContractRequest::new(ContractMethod::Put, "/api/users/1")
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "name": "Alice Updated",
            "email": "alice.updated@example.com"
        }));

    let update_user_response = ContractResponse::new(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "id": 1,
            "name": "Alice Updated",
            "email": "alice.updated@example.com"
        }));

    builder.add_interaction(
        ContractInteraction::new("update user", update_user_request, update_user_response)
            .with_provider_state("user with ID 1 exists"),
    );

    // Interaction 4: Delete user
    let delete_user_request = ContractRequest::new(ContractMethod::Delete, "/api/users/1");
    let delete_user_response = ContractResponse::new(204);

    builder.add_interaction(
        ContractInteraction::new("delete user", delete_user_request, delete_user_response)
            .with_provider_state("user with ID 1 exists"),
    );

    // Build the contract
    let contract = builder.build();

    println!(
        "   ✅ Contract created with {} interactions",
        contract.interactions.len()
    );
    println!();

    // 2. Save the contract
    println!("2. Saving Contract:");
    let contracts_dir = PathBuf::from("./pacts");
    let manager = ContractManager::new(contracts_dir.clone());

    match manager.save(&contract) {
        Ok(()) => {
            println!("   ✅ Contract saved to: {:?}", contracts_dir);
            println!("   File: frontend-userapi.json");
        }
        Err(e) => {
            println!("   ⚠️  Could not save contract: {}", e);
        }
    }

    println!();

    // 3. Load the contract (provider side)
    println!("3. Loading Contract (Provider Side):");
    match manager.load("Frontend", "UserAPI") {
        Ok(loaded_contract) => {
            println!("   ✅ Contract loaded");
            println!("   Consumer: {}", loaded_contract.consumer.name);
            println!("   Provider: {}", loaded_contract.provider.name);
            println!("   Interactions: {}", loaded_contract.interactions.len());
        }
        Err(e) => {
            println!("   ⚠️  Could not load contract: {}", e);
            println!("       (This is expected if the contract wasn't saved)");
        }
    }

    println!();

    // 4. Verify contract interactions
    println!("4. Verifying Contract:");

    // Simulate provider responses
    let actual_get_response = ContractResponse::new(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "id": 1,
            "name": "Alice",
            "email": "alice@example.com"
        }));

    let actual_create_response = ContractResponse::new(201)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "id": 2,
            "name": "Bob",
            "email": "bob@example.com"
        }));

    let actual_update_response = ContractResponse::new(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::json!({
            "id": 1,
            "name": "Alice Updated",
            "email": "alice.updated@example.com"
        }));

    let actual_delete_response = ContractResponse::new(204);

    // Verify each interaction
    let verification_results = vec![
        (contract.interactions[0].clone(), actual_get_response),
        (contract.interactions[1].clone(), actual_create_response),
        (contract.interactions[2].clone(), actual_update_response),
        (contract.interactions[3].clone(), actual_delete_response),
    ];

    for (interaction, actual) in verification_results.iter() {
        print!("   Verifying '{}': ", interaction.description);
        match ContractVerifier::verify_interaction(interaction, actual) {
            Ok(()) => println!("✅ PASS"),
            Err(e) => println!("❌ FAIL - {}", e),
        }
    }

    println!();

    // 5. Example of verification failure
    println!("5. Example of Verification Failure:");
    let failing_interaction = ContractInteraction::new(
        "get user with wrong response",
        ContractRequest::new(ContractMethod::Get, "/api/users/999"),
        ContractResponse::new(200).with_body(serde_json::json!({"id": 999})),
    );

    let actual_failing_response =
        ContractResponse::new(404).with_body(serde_json::json!({"error": "Not found"}));

    print!("   Verifying failing interaction: ");
    match ContractVerifier::verify_interaction(&failing_interaction, &actual_failing_response) {
        Ok(()) => println!("✅ PASS"),
        Err(e) => println!("❌ FAIL (expected)\n      Reason: {}", e),
    }

    println!();

    // 6. List all contracts
    println!("6. Listing All Contracts:");
    match manager.list() {
        Ok(contracts) => {
            if contracts.is_empty() {
                println!("   No contracts found");
            } else {
                for (consumer, provider) in contracts {
                    println!("   - {} → {}", consumer, provider);
                }
            }
        }
        Err(e) => {
            println!("   ⚠️  Could not list contracts: {}", e);
        }
    }

    println!();
    println!("=== Contract Testing Complete ===\n");
    println!("💡 Contract Testing Workflow:");
    println!("   1. Consumer creates contract with expected interactions");
    println!("   2. Consumer saves contract to shared location");
    println!("   3. Provider loads contract");
    println!("   4. Provider verifies each interaction");
    println!("   5. CI/CD fails if contract is broken");
    println!();
    println!("💡 Benefits:");
    println!("   - No need for integration environment");
    println!("   - Fast feedback on API changes");
    println!("   - Consumer-driven API design");
    println!("   - Version compatibility tracking");
    println!();

    Ok(())
}

//! Macro Utilities Example
//!
//! Demonstrates the various utility macros available in Armature.

use armature_core::HttpResponse;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== Armature Macro Utilities Example ===\n");

    // 1. JSON Response Macros
    println!("1. JSON Response Macros:");
    println!("   Creating JSON responses with macros...\n");

    // ok_json! equivalent - Quick 200 OK JSON response
    let success_response = HttpResponse::ok().with_json(&json!({
        "message": "User created successfully",
        "id": 123,
        "name": "Alice"
    }));

    match success_response {
        Ok(resp) => {
            println!(
                "   ✅ ok_json! created response with status {}",
                resp.status
            );
            let body_str = String::from_utf8_lossy(&resp.body);
            println!("      Body: {}", body_str);
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // created_json! equivalent - 201 Created response
    let created_response = HttpResponse::created().with_json(&json!({
        "id": 456,
        "status": "created"
    }));

    match created_response {
        Ok(resp) => {
            println!(
                "   ✅ created_json! created response with status {}",
                resp.status
            );
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // json_response! equivalent with custom status
    let custom_response = HttpResponse::new(202).with_json(&json!({
        "message": "Accepted for processing"
    }));

    match custom_response {
        Ok(resp) => {
            println!(
                "   ✅ json_response! created response with status {}",
                resp.status
            );
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // 2. Error Response Macros
    println!("2. Error Response Macros:");

    let error1 = HttpResponse::bad_request().with_json(&json!({
        "error": "Invalid email format",
        "status": 400
    }));
    match error1 {
        Ok(resp) => {
            println!(
                "   ✅ bad_request! created error with status {}",
                resp.status
            );
            let body_str = String::from_utf8_lossy(&resp.body);
            println!("      Body: {}", body_str);
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    let error2 = HttpResponse::not_found().with_json(&json!({
        "error": format!("User {} not found", 999),
        "status": 404
    }));
    match error2 {
        Ok(resp) => {
            println!("   ✅ not_found! created error with status {}", resp.status);
            let body_str = String::from_utf8_lossy(&resp.body);
            println!("      Body: {}", body_str);
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    // 3. JSON Object Builder
    println!("3. JSON Object Builder:");

    let user_data = json!({
        "id": 789,
        "name": "Bob",
        "email": "bob@example.com",
        "active": true,
    });

    println!("   ✅ JSON object created:");
    println!("      {}", serde_json::to_string_pretty(&user_data)?);
    println!();

    // 4. Paginated Response
    println!("4. Paginated Response:");

    let users = vec![
        json!({"id": 1, "name": "User 1"}),
        json!({"id": 2, "name": "User 2"}),
        json!({"id": 3, "name": "User 3"}),
    ];

    let paginated = HttpResponse::ok().with_json(&json!({
        "data": users,
        "pagination": {
            "page": 1,
            "total": 50,
            "per_page": users.len(),
        }
    }));
    match paginated {
        Ok(resp) => {
            println!("   ✅ paginated_response! created pagination:");
            let body_str = String::from_utf8_lossy(&resp.body);
            println!("      {}", body_str);
        }
        Err(e) => println!("   ❌ Error: {}", e),
    }
    println!();

    println!("=== Macro Utilities Example Complete ===\n");
    println!("💡 Available Macro Categories:");
    println!();
    println!("   📦 Declarative Response Macros (armature-macros):");
    println!("      • ok_json!() - 200 OK JSON response");
    println!("      • created_json!() - 201 Created JSON response");
    println!("      • json_response!(status, data) - Custom status JSON");
    println!("      • bad_request!(msg) - 400 Bad Request error");
    println!("      • not_found!(msg) - 404 Not Found error");
    println!("      • internal_error!(msg) - 500 Internal Server Error");
    println!();
    println!("   📦 Procedural Response Macros (armature-macros-utils):");
    println!("      • json!(data) - JSON proc macro");
    println!("      • html!(content) - HTML proc macro");
    println!("      • text!(content) - Text proc macro");
    println!("      • redirect!(url) - Redirect proc macro");
    println!();
    println!("   🔍 Parameter Extraction:");
    println!("      • path_param!(req, \"id\") - Extract path parameter");
    println!("      • query_param!(req, \"page\") - Extract query parameter");
    println!("      • header!(req, \"Auth\") - Extract header");
    println!("      • path_params!(req, \"id\": i64, \"slug\": String) - Multiple params");
    println!();
    println!("   ✅ Validation:");
    println!("      • validation_error!(msg) - Create validation error");
    println!("      • guard!(condition, msg) - Guard with error");
    println!();
    println!("   📄 Utilities:");
    println!("      • json_object! {{ }} - Build JSON objects");
    println!("      • paginated_response!(data, page, total) - Pagination");
    println!("      • log_error!(msg) - Log and return error");
    println!();
    println!("💡 Benefits:");
    println!("   ✓ Reduces boilerplate code");
    println!("   ✓ Type-safe parameter extraction");
    println!("   ✓ Consistent error formatting");
    println!("   ✓ Easier to read and maintain");
    println!("   ✓ Compile-time validation");
    println!();

    Ok(())
}

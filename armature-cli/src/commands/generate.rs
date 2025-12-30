//! Code generation commands.

use colored::Colorize;

use crate::error::{CliError, CliResult};
use crate::generators::{NameCases, ensure_dir, get_src_dir, update_mod_file, write_file};
use crate::templates::{ComponentData, ControllerData, ModuleData, TemplateRegistry};

/// Generate a controller.
pub async fn controller(name: &str, crud: bool, skip_tests: bool) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;
    let controllers_dir = src_dir.join("controllers");
    ensure_dir(&controllers_dir)?;

    let templates = TemplateRegistry::new();

    // Determine base path (handle nested paths like "api/users")
    let base_path = if name.contains('/') {
        name.to_string()
    } else {
        names.kebab.clone()
    };

    let data = ControllerData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        name_kebab: names.kebab.clone(),
        base_path,
    };

    // Generate controller file
    let template_name = if crud {
        "controller_crud"
    } else {
        "controller"
    };
    let controller_content = templates
        .render(template_name, &data)
        .map_err(CliError::Template)?;

    let controller_file = controllers_dir.join(format!("{}.rs", names.snake));
    write_file(&controller_file, &controller_content, false)?;

    println!(
        "  {} {}",
        "CREATE".green().bold(),
        controller_file.display()
    );

    // Update mod.rs
    update_mod_file(&controllers_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        controllers_dir.join("mod.rs").display()
    );

    // Generate test file
    if !skip_tests {
        let test_content = templates
            .render("controller_test", &data)
            .map_err(CliError::Template)?;

        let tests_dir = controllers_dir.join("tests");
        ensure_dir(&tests_dir)?;

        let test_file = tests_dir.join(format!("{}_test.rs", names.snake));
        write_file(&test_file, &test_content, false)?;

        println!("  {} {}", "CREATE".green().bold(), test_file.display());
    }

    println!(
        "\n{} Generated {}Controller{}",
        "✓".green().bold(),
        names.pascal,
        if crud { " with CRUD endpoints" } else { "" }
    );

    Ok(())
}

/// Generate a module.
pub async fn module(
    name: &str,
    controllers: Option<&str>,
    providers: Option<&str>,
) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;

    let templates = TemplateRegistry::new();

    let controller_list: Vec<String> = controllers
        .map(|s| {
            s.split(',')
                .map(|c| c.trim().to_string())
                .filter(|c| !c.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let provider_list: Vec<String> = providers
        .map(|s| {
            s.split(',')
                .map(|p| p.trim().to_string())
                .filter(|p| !p.is_empty())
                .collect()
        })
        .unwrap_or_default();

    let data = ModuleData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        controllers: controller_list.iter().map(|c| c.to_string()).collect(),
        providers: provider_list.iter().map(|p| p.to_string()).collect(),
        controller_list: controller_list
            .iter()
            .map(|c| format!("{}Controller", heck::AsPascalCase(c)))
            .collect::<Vec<_>>()
            .join(", "),
        provider_list: provider_list
            .iter()
            .map(|p| format!("{}Service", heck::AsPascalCase(p)))
            .collect::<Vec<_>>()
            .join(", "),
    };

    let module_content = templates
        .render("module", &data)
        .map_err(CliError::Template)?;

    // Create module directory
    let module_dir = src_dir.join(&names.snake);
    ensure_dir(&module_dir)?;

    let module_file = module_dir.join("mod.rs");
    write_file(&module_file, &module_content, false)?;

    println!("  {} {}", "CREATE".green().bold(), module_file.display());

    // Update main mod.rs
    update_mod_file(&src_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        src_dir.join("mod.rs").display()
    );

    println!("\n{} Generated {}Module", "✓".green().bold(), names.pascal);

    Ok(())
}

/// Generate middleware.
pub async fn middleware(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("middleware", name, skip_tests).await
}

/// Generate a guard.
pub async fn guard(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("guard", name, skip_tests).await
}

/// Generate a service.
pub async fn service(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("service", name, skip_tests).await
}

/// Generate a complete resource (controller + service + module).
pub async fn resource(name: &str, crud: bool) -> CliResult<()> {
    println!(
        "  {} Generating resource: {}",
        "→".cyan().bold(),
        name.cyan()
    );
    println!();

    // Generate service
    println!("  {} Generating service...", "1/3".dimmed());
    service(name, false).await?;
    println!();

    // Generate controller
    println!("  {} Generating controller...", "2/3".dimmed());
    controller(name, crud, false).await?;
    println!();

    // Generate module
    println!("  {} Generating module...", "3/3".dimmed());
    module(name, Some(name), Some(name)).await?;

    println!(
        "\n{} Resource {} generated successfully!",
        "✓".green().bold(),
        name.green()
    );
    println!(
        "  {} Don't forget to import the module in your main.rs",
        "→".yellow()
    );

    Ok(())
}

/// Generate a repository.
pub async fn repository(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("repository", name, skip_tests).await
}

/// Generate DTOs (Data Transfer Objects).
pub async fn dto(name: &str) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;
    let dto_dir = src_dir.join("dto");
    ensure_dir(&dto_dir)?;

    let templates = TemplateRegistry::new();

    let data = ComponentData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        name_kebab: names.kebab.clone(),
    };

    let content = templates
        .render("dto", &data)
        .map_err(CliError::Template)?;

    let file_path = dto_dir.join(format!("{}.rs", names.snake));
    write_file(&file_path, &content, false)?;

    println!("  {} {}", "CREATE".green().bold(), file_path.display());

    update_mod_file(&dto_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        dto_dir.join("mod.rs").display()
    );

    println!("\n{} Generated {}Dto", "✓".green().bold(), names.pascal);
    Ok(())
}

/// Generate a WebSocket handler.
pub async fn websocket(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("websocket", name, skip_tests).await
}

/// Generate a GraphQL resolver.
pub async fn graphql_resolver(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("graphql_resolver", name, skip_tests).await
}

/// Generate a background job.
pub async fn job(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("job", name, skip_tests).await
}

/// Generate an event handler.
pub async fn event_handler(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("event_handler", name, skip_tests).await
}

/// Generate an interceptor.
pub async fn interceptor(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("interceptor", name, skip_tests).await
}

/// Generate a validation pipe.
pub async fn pipe(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("pipe", name, skip_tests).await
}

/// Generate an exception filter.
pub async fn exception_filter(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("exception_filter", name, skip_tests).await
}

/// Generate a configuration module.
pub async fn config(name: &str) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;
    let config_dir = src_dir.join("config");
    ensure_dir(&config_dir)?;

    let templates = TemplateRegistry::new();

    let data = crate::templates::ComponentData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        name_kebab: names.kebab.clone(),
    };

    let content = templates
        .render("config", &data)
        .map_err(CliError::Template)?;

    let file_path = config_dir.join(format!("{}.rs", names.snake));
    write_file(&file_path, &content, false)?;

    println!("  {} {}", "CREATE".green().bold(), file_path.display());

    update_mod_file(&config_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        config_dir.join("mod.rs").display()
    );

    println!("\n{} Generated {}Config", "✓".green().bold(), names.pascal);
    Ok(())
}

/// Generate a database entity.
pub async fn entity(name: &str) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;
    let entities_dir = src_dir.join("entities");
    ensure_dir(&entities_dir)?;

    let templates = TemplateRegistry::new();

    let data = ComponentData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        name_kebab: names.kebab.clone(),
    };

    let content = templates
        .render("entity", &data)
        .map_err(CliError::Template)?;

    let file_path = entities_dir.join(format!("{}.rs", names.snake));
    write_file(&file_path, &content, false)?;

    println!("  {} {}", "CREATE".green().bold(), file_path.display());

    update_mod_file(&entities_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        entities_dir.join("mod.rs").display()
    );

    println!("\n{} Generated {} entity", "✓".green().bold(), names.pascal);
    Ok(())
}

/// Generate a scheduled task.
pub async fn scheduler(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("scheduler", name, skip_tests).await
}

/// Generate a cache service.
pub async fn cache_service(name: &str, skip_tests: bool) -> CliResult<()> {
    generate_component("cache_service", name, skip_tests).await
}

/// Generate an API client.
pub async fn api_client(name: &str) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;
    let clients_dir = src_dir.join("clients");
    ensure_dir(&clients_dir)?;

    let templates = TemplateRegistry::new();

    let data = ComponentData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        name_kebab: names.kebab.clone(),
    };

    let content = templates
        .render("api_client", &data)
        .map_err(CliError::Template)?;

    let file_path = clients_dir.join(format!("{}.rs", names.snake));
    write_file(&file_path, &content, false)?;

    println!("  {} {}", "CREATE".green().bold(), file_path.display());

    update_mod_file(&clients_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        clients_dir.join("mod.rs").display()
    );

    println!("\n{} Generated {}Client", "✓".green().bold(), names.pascal);
    Ok(())
}

/// Generate a health check controller.
pub async fn health_controller() -> CliResult<()> {
    let src_dir = get_src_dir()?;
    let controllers_dir = src_dir.join("controllers");
    ensure_dir(&controllers_dir)?;

    let templates = TemplateRegistry::new();

    let data = ComponentData {
        name_pascal: "Health".to_string(),
        name_snake: "health".to_string(),
        name_kebab: "health".to_string(),
    };

    let content = templates
        .render("health_controller", &data)
        .map_err(CliError::Template)?;

    let file_path = controllers_dir.join("health.rs");
    write_file(&file_path, &content, false)?;

    println!("  {} {}", "CREATE".green().bold(), file_path.display());

    update_mod_file(&controllers_dir, "health")?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        controllers_dir.join("mod.rs").display()
    );

    println!("\n{} Generated HealthController", "✓".green().bold());
    Ok(())
}

/// Generic component generator for middleware, guards, services, and more.
async fn generate_component(component_type: &str, name: &str, skip_tests: bool) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;

    let dir_name = match component_type {
        "middleware" => "middleware",
        "guard" => "guards",
        "service" => "services",
        "repository" => "repositories",
        "websocket" => "websockets",
        "graphql_resolver" => "graphql",
        "job" => "jobs",
        "event_handler" => "events",
        "interceptor" => "interceptors",
        "pipe" => "pipes",
        "exception_filter" => "filters",
        "scheduler" => "tasks",
        "cache_service" => "cache",
        _ => {
            return Err(CliError::InvalidArgument(format!(
                "Unknown component type: {}",
                component_type
            )));
        }
    };

    let component_dir = src_dir.join(dir_name);
    ensure_dir(&component_dir)?;

    let templates = TemplateRegistry::new();

    let data = ComponentData {
        name_pascal: names.pascal.clone(),
        name_snake: names.snake.clone(),
        name_kebab: names.kebab.clone(),
    };

    // Generate main file
    let content = templates
        .render(component_type, &data)
        .map_err(CliError::Template)?;

    let file_path = component_dir.join(format!("{}.rs", names.snake));
    write_file(&file_path, &content, false)?;

    println!("  {} {}", "CREATE".green().bold(), file_path.display());

    // Update mod.rs
    update_mod_file(&component_dir, &names.snake)?;
    println!(
        "  {} {}",
        "UPDATE".yellow().bold(),
        component_dir.join("mod.rs").display()
    );

    // Generate test file
    if !skip_tests {
        let test_template = format!("{}_test", component_type);
        let test_content = templates
            .render(&test_template, &data)
            .map_err(CliError::Template)?;

        let tests_dir = component_dir.join("tests");
        ensure_dir(&tests_dir)?;

        let test_file = tests_dir.join(format!("{}_test.rs", names.snake));
        write_file(&test_file, &test_content, false)?;

        println!("  {} {}", "CREATE".green().bold(), test_file.display());
    }

    let type_name = match component_type {
        "middleware" => "Middleware",
        "guard" => "Guard",
        "service" => "Service",
        "repository" => "Repository",
        "websocket" => "WebSocket",
        "graphql_resolver" => "Resolver",
        "job" => "Job",
        "event_handler" => "EventHandler",
        "interceptor" => "Interceptor",
        "pipe" => "Pipe",
        "exception_filter" => "ExceptionFilter",
        "scheduler" => "Task",
        "cache_service" => "CacheService",
        _ => "Component",
    };

    println!(
        "\n{} Generated {}{}",
        "✓".green().bold(),
        names.pascal,
        type_name
    );

    Ok(())
}

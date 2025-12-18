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

/// Generic component generator for middleware, guards, and services.
async fn generate_component(component_type: &str, name: &str, skip_tests: bool) -> CliResult<()> {
    let names = NameCases::from(name);
    let src_dir = get_src_dir()?;

    let dir_name = match component_type {
        "middleware" => "middleware",
        "guard" => "guards",
        "service" => "services",
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

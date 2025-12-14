// Tests for dependency injection system

use armature_core::{Container, Provider};

#[derive(Clone)]
struct ServiceA {
    value: String,
}


#[derive(Clone)]
struct ServiceB {
    service_a: ServiceA,
    name: String,
}


#[test]
fn test_register_and_resolve_service() {
    let container = Container::new();

    let service = ServiceA {
        value: "test".to_string(),
    };

    container.register(service);

    let resolved = container.resolve::<ServiceA>().unwrap();
    assert_eq!(resolved.value, "test");
}

#[test]
fn test_service_with_dependency() {
    let container = Container::new();

    // Register dependency first
    let service_a = ServiceA {
        value: "dependency".to_string(),
    };
    container.register(service_a.clone());

    // Register service with dependency
    let service_b = ServiceB {
        service_a,
        name: "dependent".to_string(),
    };
    container.register(service_b);

    // Resolve and check
    let resolved_b = container.resolve::<ServiceB>().unwrap();
    assert_eq!(resolved_b.name, "dependent");
    assert_eq!(resolved_b.service_a.value, "dependency");
}

#[test]
fn test_singleton_behavior() {
    let container = Container::new();

    let service = ServiceA {
        value: "singleton".to_string(),
    };
    container.register(service);

    // Resolve twice
    let resolved1 = container.resolve::<ServiceA>().unwrap();
    let resolved2 = container.resolve::<ServiceA>().unwrap();

    // Both should have the same value (singleton)
    assert_eq!(resolved1.value, resolved2.value);
}

#[test]
fn test_multiple_services() {
    let container = Container::new();

    let service_a = ServiceA {
        value: "A".to_string(),
    };
    container.register(service_a.clone());

    let service_b = ServiceB {
        service_a,
        name: "B".to_string(),
    };
    container.register(service_b);

    // Both should be resolvable
    let resolved_a = container.resolve::<ServiceA>().unwrap();
    let resolved_b = container.resolve::<ServiceB>().unwrap();

    assert_eq!(resolved_a.value, "A");
    assert_eq!(resolved_b.name, "B");
}

#[test]
fn test_container_has() {
    let container = Container::new();

    assert!(!container.has::<ServiceA>());

    container.register(ServiceA {
        value: "test".to_string(),
    });

    assert!(container.has::<ServiceA>());
    assert!(!container.has::<ServiceB>());
}

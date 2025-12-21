use armature_core::Container;

#[derive(Clone)]
struct TestService {
    name: String,
}

#[test]
fn test_register_and_resolve() {
    let container = Container::new();

    let service = TestService {
        name: "test".to_string(),
    };

    container.register(service);

    let resolved = container.resolve::<TestService>().unwrap();
    assert_eq!(resolved.name, "test");
}

#[test]
fn test_resolve_nonexistent() {
    let container = Container::new();
    let result = container.resolve::<TestService>();
    assert!(result.is_err());
}

#[test]
fn test_has_provider() {
    let container = Container::new();

    assert!(!container.has::<TestService>());

    container.register(TestService {
        name: "test".to_string(),
    });

    assert!(container.has::<TestService>());
}

#[test]
fn test_singleton_behavior() {
    let container = Container::new();

    container.register(TestService {
        name: "original".to_string(),
    });

    let resolved1 = container.resolve::<TestService>().unwrap();
    let resolved2 = container.resolve::<TestService>().unwrap();

    // Both should reference the same instance
    assert_eq!(resolved1.name, resolved2.name);
}

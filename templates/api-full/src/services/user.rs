//! User service

use crate::models::{User, UserRole};
use armature::Provider;
use chrono::Utc;
use std::any::Any;
use std::collections::HashMap;
use std::sync::RwLock;
use uuid::Uuid;

pub struct UserService {
    users: RwLock<HashMap<Uuid, User>>,
}

impl UserService {
    pub fn new() -> Self {
        let mut users = HashMap::new();

        // Create a default admin user
        let admin_id = Uuid::new_v4();
        users.insert(
            admin_id,
            User {
                id: admin_id,
                email: "admin@example.com".to_string(),
                // password: admin123 (hashed with default secret)
                password_hash: "5d7c7c3e0f0b6a0b".to_string(),
                name: "Admin User".to_string(),
                role: UserRole::Admin,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
        );

        Self {
            users: RwLock::new(users),
        }
    }

    pub fn find_by_id(&self, id: Uuid) -> Option<User> {
        self.users.read().unwrap().get(&id).cloned()
    }

    pub fn find_by_email(&self, email: &str) -> Option<User> {
        self.users
            .read()
            .unwrap()
            .values()
            .find(|u| u.email == email)
            .cloned()
    }

    pub fn find_all(&self) -> Vec<User> {
        self.users.read().unwrap().values().cloned().collect()
    }

    pub fn create(&self, email: String, password_hash: String, name: String) -> User {
        let id = Uuid::new_v4();
        let now = Utc::now();

        let user = User {
            id,
            email,
            password_hash,
            name,
            role: UserRole::User,
            created_at: now,
            updated_at: now,
        };

        self.users.write().unwrap().insert(id, user.clone());
        user
    }

    pub fn update(&self, id: Uuid, name: Option<String>) -> Option<User> {
        let mut users = self.users.write().unwrap();

        if let Some(user) = users.get_mut(&id) {
            if let Some(n) = name {
                user.name = n;
            }
            user.updated_at = Utc::now();
            return Some(user.clone());
        }

        None
    }

    pub fn delete(&self, id: Uuid) -> bool {
        self.users.write().unwrap().remove(&id).is_some()
    }

    pub fn count(&self) -> usize {
        self.users.read().unwrap().len()
    }

    pub fn email_exists(&self, email: &str) -> bool {
        self.find_by_email(email).is_some()
    }
}

impl Default for UserService {
    fn default() -> Self {
        Self::new()
    }
}

// Provider is automatically implemented via blanket impl


// src/test_files/rust/sample_service.rs
//
// A representative Rust file for integration testing.
// Covers: structs, enums, traits, impl blocks, methods, fields,
// derives, visibility, unsafe, async, generics.

use std::collections::HashMap;
use tokio::sync::RwLock;

pub mod models {
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct User {
        pub id: u64,
        pub name: String,
        pub email: String,
        pub role: Role,
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Role {
        Admin,
        Editor,
        Viewer,
    }
}

pub trait Repository<T> {
    fn find_by_id(&self, id: u64) -> Option<T>;
    fn save(&mut self, entity: T) -> Result<(), String>;
    fn delete(&self, id: u64) -> bool;
}

pub struct UserService {
    users: HashMap<u64, models::User>,
    cache: RwLock<Vec<u64>>,
}

impl UserService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            cache: RwLock::new(Vec::new()),
        }
    }

    pub async fn get_user(&self, id: u64) -> Option<&models::User> {
        self.users.get(&id)
    }

    pub fn create_user(&mut self, user: models::User) -> u64 {
        let id = user.id;
        self.users.insert(id, user);
        id
    }

    pub fn delete_user(&mut self, id: u64) -> bool {
        self.users.remove(&id).is_some()
    }
}

impl Repository<models::User> for UserService {
    fn find_by_id(&self, id: u64) -> Option<models::User> {
        self.users.get(&id).cloned()
    }

    fn save(&mut self, user: models::User) -> Result<(), String> {
        self.users.insert(user.id, user);
        Ok(())
    }

    fn delete(&self, id: u64) -> bool {
        self.users.contains_key(&id)
    }
}

pub trait Processor {
    fn process(&self, data: &str) -> Result<String, String>;
}

pub struct DataProcessor {
    buffer: Vec<u8>,
}

impl DataProcessor {
    pub fn new(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }
}

impl Processor for DataProcessor {
    fn process(&self, data: &str) -> Result<String, String> {
        if data.is_empty() {
            return Err("empty input".to_string());
        }
        Ok(data.to_uppercase())
    }
}

unsafe impl Send for DataProcessor {}

type UserId = u64;
type UserMap = HashMap<UserId, models::User>;
Q: I am using a  text-based custom database that contains , for example, users.txt:
userId>>>username>>>password>>>|||
userId>>>username>>>password>>>|||
userId>>>username>>>password>>>|||]]]
So >>> is field delimeter, ||| is record delimeter and ]]] is ending marker.
Same for forum_topics.txt, chat_messages.txt, etc.

I have currently through user.rs, forum.rs, chat.rs this style of 
read_ functions and write_:
```rust
fn read_users() -> Result<Vec<User>, std::io::Error> {
    let mut users = Vec::new();
    let users_file_path = format!(
        "{}/users.{}",
        storage::STORAGE_DIR_NAME,
        storage::STORAGE_FILE_EXTENSION
    );

    let content = std::fs::read_to_string(users_file_path)?;

    // Remove the ending marker
    let content = content.trim_end_matches(storage::FILE_ENDING_MARKER).trim();

    // Split by >>
    if !content.is_empty() {
        for entry in content.split(storage::RECORD_SEPARATOR) {
            let parts: Vec<&str> = entry.split(storage::FIELD_SEPARATOR).collect();
            if parts.len() == 4 {
                let id = parts[0].to_string();
                let username = parts[1].to_string();
                let password = parts[2].to_string();
                let bio = parts[3].to_string();

                let id = storage::unescape(id);
                let username = storage::unescape(username);
                let password = storage::unescape(password);
                let bio = storage::unescape(bio);

                users.push(User {
                    id,
                    username,
                    password,
                    bio,
                });
            }
        }
    }

    Ok(users)
}

fn write_users(users: Vec<User>) -> Result<(), std::io::Error> {
    let mut content = String::new();

    for user in users {
        let entry = format!(
            "{}{d}{}{d}{}{d}{}",
            storage::escape(user.id),
            storage::escape(user.username),
            storage::escape(user.password),
            storage::escape(user.bio),
            d = storage::FIELD_SEPARATOR
        );
        if !content.is_empty() {
            content.push_str(storage::RECORD_SEPARATOR);
        }
        content.push_str(&entry);
    }

    // Add storage_init::RECORD_SEPARATOR and storage_init::FILE_ENDING_MARKER
    content.push_str(
        format!(
            "{}{}",
            storage::RECORD_SEPARATOR,
            storage::FILE_ENDING_MARKER
        )
        .as_str(),
    );

    std::fs::write(
        storage::get_file_path("users".to_string()).unwrap(),
        content,
    )
}
```
escape and unescape escape and unescape the delimeters.

How do I create a more generic storage::get_contents("users") and storage::write_contents(users, "users") for all files?


A: You can create a generic storage module with traits and functions that handle the serialization/deserialization. Here's a comprehensive approach:

## 1. First, define traits for serializable entities:

```rust
// storage.rs
use std::path::Path;

pub const STORAGE_DIR_NAME: &str = "data";
pub const STORAGE_FILE_EXTENSION: &str = "txt";
pub const FIELD_SEPARATOR: &str = ">>>";
pub const RECORD_SEPARATOR: &str = "|||";
pub const FILE_ENDING_MARKER: &str = "]]]";

pub trait Storable: Sized {
    /// Serialize the item to a string representation
    fn serialize(&self) -> String;
    
    /// Deserialize from a string slice
    fn deserialize(data: &str) -> Option<Self>;
    
    /// Get the field separator (can be overridden if needed)
    fn field_separator() -> &'static str {
        FIELD_SEPARATOR
    }
    
    /// Get the record separator (can be overridden if needed)
    fn record_separator() -> &'static str {
        RECORD_SEPARATOR
    }
    
    /// Get the file ending marker (can be overridden if needed)
    fn file_ending_marker() -> &'static str {
        FILE_ENDING_MARKER
    }
}

pub trait StorageOperations {
    /// Get the file name for this type (without extension)
    fn file_name() -> String;
}
```

## 2. Create a generic storage module with helper functions:

```rust
// storage.rs (continued)
use std::fs;

/// Escape special characters in the data
pub fn escape(data: String) -> String {
    data.replace(FIELD_SEPARATOR, "\\>>>")
        .replace(RECORD_SEPARATOR, "\\|||")
        .replace(FILE_ENDING_MARKER, "\\]]]")
}

/// Unescape special characters in the data
pub fn unescape(data: String) -> String {
    data.replace("\\>>>", FIELD_SEPARATOR)
        .replace("\\|||", RECORD_SEPARATOR)
        .replace("\\]]]", FILE_ENDING_MARKER)
}

/// Get the full file path for a given entity type
pub fn get_file_path<T: StorageOperations>() -> String {
    format!(
        "{}/{}.{}",
        STORAGE_DIR_NAME,
        T::file_name(),
        STORAGE_FILE_EXTENSION
    )
}

/// Generic function to read all items of type T
pub fn read_contents<T>() -> Result<Vec<T>, std::io::Error>
where
    T: Storable + StorageOperations,
{
    let file_path = get_file_path::<T>();
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist yet, return empty vector
            return Ok(Vec::new());
        }
        Err(e) => return Err(e),
    };

    let mut items = Vec::new();
    
    // Remove the ending marker
    let ending_marker = T::file_ending_marker();
    let content = content.trim_end_matches(ending_marker).trim();

    if !content.is_empty() {
        let record_separator = T::record_separator();
        for entry in content.split(record_separator) {
            if let Some(item) = T::deserialize(entry) {
                items.push(item);
            }
        }
    }

    Ok(items)
}

/// Generic function to write items of type T
pub fn write_contents<T>(items: &[T]) -> Result<(), std::io::Error>
where
    T: Storable + StorageOperations,
{
    let mut content = String::new();
    let record_separator = T::record_separator();
    let ending_marker = T::file_ending_marker();

    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            content.push_str(record_separator);
        }
        content.push_str(&item.serialize());
    }

    // Add the ending marker
    if !content.is_empty() {
        content.push_str(record_separator);
    }
    content.push_str(ending_marker);

    let file_path = get_file_path::<T>();
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file_path, content)
}

/// Function to get items by a specific file name (for flexibility)
pub fn read_contents_by_name<T>(file_name: &str) -> Result<Vec<T>, std::io::Error>
where
    T: Storable,
{
    let file_path = format!(
        "{}/{}.{}",
        STORAGE_DIR_NAME,
        file_name,
        STORAGE_FILE_EXTENSION
    );
    
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(e),
    };

    let mut items = Vec::new();
    let content = content.trim_end_matches(FILE_ENDING_MARKER).trim();

    if !content.is_empty() {
        for entry in content.split(RECORD_SEPARATOR) {
            if let Some(item) = T::deserialize(entry) {
                items.push(item);
            }
        }
    }

    Ok(items)
}

/// Function to write items by a specific file name
pub fn write_contents_by_name<T>(items: &[T], file_name: &str) -> Result<(), std::io::Error>
where
    T: Storable,
{
    let mut content = String::new();

    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            content.push_str(RECORD_SEPARATOR);
        }
        content.push_str(&item.serialize());
    }

    if !content.is_empty() {
        content.push_str(RECORD_SEPARATOR);
    }
    content.push_str(FILE_ENDING_MARKER);

    let file_path = format!(
        "{}/{}.{}",
        STORAGE_DIR_NAME,
        file_name,
        STORAGE_FILE_EXTENSION
    );
    
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file_path, content)
}
```

## 3. Implement these traits for your User struct:

```rust
// user.rs
use crate::storage::{self, Storable, StorageOperations};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub bio: String,
}

impl Storable for User {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}",
            storage::escape(self.id.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.username.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.password.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.bio.clone())
        )
    }

    fn deserialize(data: &str) -> Option<Self> {
        let parts: Vec<&str> = data.split(storage::FIELD_SEPARATOR).collect();
        if parts.len() == 4 {
            Some(User {
                id: storage::unescape(parts[0].to_string()),
                username: storage::unescape(parts[1].to_string()),
                password: storage::unescape(parts[2].to_string()),
                bio: storage::unescape(parts[3].to_string()),
            })
        } else {
            None
        }
    }
}

impl StorageOperations for User {
    fn file_name() -> String {
        "users".to_string()
    }
}

// Helper functions that use the generic storage
pub fn read_users() -> Result<Vec<User>, std::io::Error> {
    storage::read_contents::<User>()
}

pub fn write_users(users: &[User]) -> Result<(), std::io::Error> {
    storage::write_contents::<User>(users)
}
```

## 4. Example for other entities (ForumTopic):

```rust
// forum.rs
use crate::storage::{self, Storable, StorageOperations};

#[derive(Debug, Clone)]
pub struct ForumTopic {
    pub id: String,
    pub title: String,
    pub content: String,
    pub author_id: String,
    pub created_at: String,
}

impl Storable for ForumTopic {
    fn serialize(&self) -> String {
        format!(
            "{}{}{}{}{}{}{}{}{}{}{}",
            storage::escape(self.id.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.title.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.content.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.author_id.clone()),
            storage::FIELD_SEPARATOR,
            storage::escape(self.created_at.clone()),
            storage::FIELD_SEPARATOR,
            "0" // Example: additional field like "is_locked"
        )
    }

    fn deserialize(data: &str) -> Option<Self> {
        let parts: Vec<&str> = data.split(storage::FIELD_SEPARATOR).collect();
        if parts.len() >= 5 {
            Some(ForumTopic {
                id: storage::unescape(parts[0].to_string()),
                title: storage::unescape(parts[1].to_string()),
                content: storage::unescape(parts[2].to_string()),
                author_id: storage::unescape(parts[3].to_string()),
                created_at: storage::unescape(parts[4].to_string()),
            })
        } else {
            None
        }
    }
}

impl StorageOperations for ForumTopic {
    fn file_name() -> String {
        "forum_topics".to_string()
    }
}

pub fn read_topics() -> Result<Vec<ForumTopic>, std::io::Error> {
    storage::read_contents::<ForumTopic>()
}

pub fn write_topics(topics: &[ForumTopic]) -> Result<(), std::io::Error> {
    storage::write_contents::<ForumTopic>(topics)
}
```

## 5. Usage examples:

```rust
// main.rs or other modules
mod storage;
mod user;
mod forum;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Using the generic functions directly
    let users: Vec<user::User> = storage::read_contents_by_name("users")?;
    
    // Or using the type-specific helper functions
    let users = user::read_users()?;
    
    // Write using generic function
    storage::write_contents_by_name(&users, "users")?;
    
    // Or using type-specific function
    user::write_users(&users)?;
    
    // For forum topics
    let topics = forum::read_topics()?;
    forum::write_topics(&topics)?;
    
    Ok(())
}
```

## Benefits of this approach:

1. **Type safety** - Each struct knows how to serialize/deserialize itself
2. **Code reuse** - No duplicate parsing logic
3. **Flexibility** - Can override separators per type if needed
4. **Error handling** - Consistent error handling across all entity types
5. **Easy to add new entities** - Just implement the two traits

## Alternative simpler approach:

If you want something even simpler without traits, you could create functions that accept closures:

```rust
pub fn read_with_parser<T, F>(file_name: &str, parser: F) -> Result<Vec<T>, std::io::Error>
where
    F: Fn(&str) -> Option<T>,
{
    // ... read file and parse each record with the parser closure
}

// Usage:
let users = storage::read_with_parser("users", |data| {
    let parts: Vec<&str> = data.split(FIELD_SEPARATOR).collect();
    if parts.len() == 4 {
        Some(User {
            id: unescape(parts[0].to_string()),
            // ...
        })
    } else {
        None
    }
})?;
```

Choose the approach that best fits your project's complexity and needs!

Q2: It might be good to do all unescapes and escapes inside the generic funcs rather than make all implementers do them

A2:
You're absolutely right! It's much cleaner to handle escaping/unescaping in the generic functions. Here's the improved approach:

## 1. Updated storage module with internal escaping:

```rust
// storage.rs
use std::fs;
use std::path::Path;

pub const STORAGE_DIR_NAME: &str = "data";
pub const STORAGE_FILE_EXTENSION: &str = "txt";
pub const FIELD_SEPARATOR: &str = ">>>";
pub const RECORD_SEPARATOR: &str = "|||";
pub const FILE_ENDING_MARKER: &str = "]]]";

/// Trait for types that can be stored in the database
pub trait Storable: Sized {
    /// Convert the item to a vector of strings (the fields)
    fn to_fields(&self) -> Vec<String>;
    
    /// Create an item from a vector of strings (the fields)
    fn from_fields(fields: Vec<String>) -> Option<Self>;
    
    /// Get the file name for this type
    fn file_name() -> &'static str;
}

/// Escape special characters in a string
fn escape_field(data: &str) -> String {
    data.replace(FIELD_SEPARATOR, "\\>>>")
        .replace(RECORD_SEPARATOR, "\\|||")
        .replace(FILE_ENDING_MARKER, "\\]]]")
}

/// Unescape special characters in a string
fn unescape_field(data: &str) -> String {
    data.replace("\\>>>", FIELD_SEPARATOR)
        .replace("\\|||", RECORD_SEPARATOR)
        .replace("\\]]]", FILE_ENDING_MARKER)
}

/// Get the full file path for a type
pub fn get_file_path<T: Storable>() -> String {
    format!(
        "{}/{}.{}",
        STORAGE_DIR_NAME,
        T::file_name(),
        STORAGE_FILE_EXTENSION
    )
}

/// Generic function to read all items of type T
pub fn read_contents<T: Storable>() -> Result<Vec<T>, std::io::Error> {
    let file_path = get_file_path::<T>();
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // File doesn't exist yet, return empty vector
            return Ok(Vec::new());
        }
        Err(e) => return Err(e),
    };

    let mut items = Vec::new();
    
    // Remove the ending marker
    let content = content.trim_end_matches(FILE_ENDING_MARKER).trim();

    if !content.is_empty() {
        for entry in content.split(RECORD_SEPARATOR) {
            // Skip empty entries (can happen if there are trailing separators)
            if entry.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = entry
                .split(FIELD_SEPARATOR)
                .map(|field| unescape_field(field.trim()))
                .collect();
            
            if let Some(item) = T::from_fields(fields) {
                items.push(item);
            } else {
                // Log error or handle malformed data
                eprintln!("Warning: Failed to parse entry: {}", entry);
            }
        }
    }

    Ok(items)
}

/// Generic function to write items of type T
pub fn write_contents<T: Storable>(items: &[T]) -> Result<(), std::io::Error> {
    let mut content = String::new();

    for (i, item) in items.iter().enumerate() {
        // Get the fields and escape them
        let escaped_fields: Vec<String> = item
            .to_fields()
            .iter()
            .map(|field| escape_field(field))
            .collect();
        
        // Join escaped fields with field separator
        let entry = escaped_fields.join(FIELD_SEPARATOR);
        
        if i > 0 {
            content.push_str(RECORD_SEPARATOR);
        }
        content.push_str(&entry);
    }

    // Add the ending marker
    if !content.is_empty() {
        content.push_str(RECORD_SEPARATOR);
    }
    content.push_str(FILE_ENDING_MARKER);

    let file_path = get_file_path::<T>();
    
    // Create directory if it doesn't exist
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file_path, content)
}

/// Helper function to read by file name (for when you don't know the type at compile time)
pub fn read_contents_by_name<T: Storable>(file_name: &str) -> Result<Vec<T>, std::io::Error> {
    let file_path = format!(
        "{}/{}.{}",
        STORAGE_DIR_NAME,
        file_name,
        STORAGE_FILE_EXTENSION
    );
    
    let content = match fs::read_to_string(&file_path) {
        Ok(content) => content,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Vec::new());
        }
        Err(e) => return Err(e),
    };

    let mut items = Vec::new();
    let content = content.trim_end_matches(FILE_ENDING_MARKER).trim();

    if !content.is_empty() {
        for entry in content.split(RECORD_SEPARATOR) {
            if entry.trim().is_empty() {
                continue;
            }
            
            let fields: Vec<String> = entry
                .split(FIELD_SEPARATOR)
                .map(|field| unescape_field(field.trim()))
                .collect();
            
            if let Some(item) = T::from_fields(fields) {
                items.push(item);
            }
        }
    }

    Ok(items)
}

/// Helper function to write by file name
pub fn write_contents_by_name<T: Storable>(items: &[T], file_name: &str) -> Result<(), std::io::Error> {
    let mut content = String::new();

    for (i, item) in items.iter().enumerate() {
        let escaped_fields: Vec<String> = item
            .to_fields()
            .iter()
            .map(|field| escape_field(field))
            .collect();
        
        let entry = escaped_fields.join(FIELD_SEPARATOR);
        
        if i > 0 {
            content.push_str(RECORD_SEPARATOR);
        }
        content.push_str(&entry);
    }

    if !content.is_empty() {
        content.push_str(RECORD_SEPARATOR);
    }
    content.push_str(FILE_ENDING_MARKER);

    let file_path = format!(
        "{}/{}.{}",
        STORAGE_DIR_NAME,
        file_name,
        STORAGE_FILE_EXTENSION
    );
    
    if let Some(parent) = Path::new(&file_path).parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(file_path, content)
}
```

## 2. Simplified User struct implementation:

```rust
// user.rs
use crate::storage::{Storable, read_contents, write_contents};

#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub bio: String,
}

impl Storable for User {
    fn to_fields(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.username.clone(),
            self.password.clone(),
            self.bio.clone(),
        ]
    }

    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() == 4 {
            Some(User {
                id: fields[0].clone(),
                username: fields[1].clone(),
                password: fields[2].clone(),
                bio: fields[3].clone(),
            })
        } else {
            None
        }
    }

    fn file_name() -> &'static str {
        "users"
    }
}

// Optional: Keep the helper functions for convenience
pub fn read_users() -> Result<Vec<User>, std::io::Error> {
    read_contents::<User>()
}

pub fn write_users(users: &[User]) -> Result<(), std::io::Error> {
    write_contents::<User>(users)
}
```

## 3. Example with ChatMessage:

```rust
// chat.rs
use crate::storage::Storable;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub id: String,
    pub user_id: String,
    pub room_id: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_edited: bool,
}

impl Storable for ChatMessage {
    fn to_fields(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.user_id.clone(),
            self.room_id.clone(),
            self.content.clone(),
            self.timestamp.to_rfc3339(),
            self.is_edited.to_string(),
        ]
    }

    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() == 6 {
            let timestamp = DateTime::parse_from_rfc3339(&fields[4])
                .ok()?
                .with_timezone(&Utc);
            let is_edited = fields[5].parse::<bool>().unwrap_or(false);
            
            Some(ChatMessage {
                id: fields[0].clone(),
                user_id: fields[1].clone(),
                room_id: fields[2].clone(),
                content: fields[3].clone(),
                timestamp,
                is_edited,
            })
        } else {
            None
        }
    }

    fn file_name() -> &'static str {
        "chat_messages"
    }
}
```

## 4. Even simpler with a derive-like macro (optional):

If you have many structs, you could create a procedural macro:

```rust
// Example of what you'd like to write:
#[derive(Storable)]
#[storage_file_name = "users"]
pub struct User {
    pub id: String,
    pub username: String,
    pub password: String,
    pub bio: String,
}
```

But for now, the manual implementation is simple enough.

## 5. Advanced: With validation in from_fields:

```rust
impl Storable for User {
    fn to_fields(&self) -> Vec<String> {
        vec![
            self.id.clone(),
            self.username.clone(),
            self.password.clone(),
            self.bio.clone(),
        ]
    }

    fn from_fields(fields: Vec<String>) -> Option<Self> {
        if fields.len() != 4 {
            return None;
        }
        
        // Validate that required fields are not empty
        if fields[0].is_empty() || fields[1].is_empty() {
            return None;
        }
        
        Some(User {
            id: fields[0].clone(),
            username: fields[1].clone(),
            password: fields[2].clone(),
            bio: fields[3].clone(),
        })
    }

    fn file_name() -> &'static str {
        "users"
    }
}
```

## Benefits of this approach:

1. **No escaping/unescaping in each struct** - It's all handled in the generic functions
2. **Clean separation of concerns** - Structs just define their fields, storage handles serialization
3. **Consistent behavior** - All types are serialized/deserialized the same way
4. **Error handling** - Malformed entries are skipped with warnings
5. **Flexible field types** - You can store any type that can be converted to/from String

## Usage:

```rust
// Read all users
let users: Vec<User> = storage::read_contents()?;

// Add a new user
let mut users = storage::read_contents::<User>()?;
users.push(User {
    id: "123".to_string(),
    username: "john".to_string(),
    password: "pass".to_string(),
    bio: "Hello".to_string(),
});
storage::write_contents(&users)?;

// Read forum topics
let topics = storage::read_contents::<ForumTopic>()?;
```

This design centralizes all the escaping logic and makes it much easier to add new entity types to your database.
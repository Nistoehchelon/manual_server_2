use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use crate::storage;

pub const FILE_ENDING_MARKER: &str = "{]]]}";
pub const RECORD_SEPARATOR: &str = "{>>>}";
pub const FIELD_SEPARATOR: &str = "{|||}";

pub const STORAGE_DIR_NAME: &str = "./txt_db";
pub const RESOURCES_PATH: &str = "./src/resources";
pub const HEADER_FILE_NAME: &str = "header.html";
pub const FOOTER_FILE_NAME: &str = "footer.html";
pub const STYLESHEET_FILE_NAME: &str = "stylesheet.css";

pub const STORAGE_FILE_NAMES: [&str; 10] = [
    "sessions",
    "session_notifications",
    "users",
    "forums",
    "forum_threads",
    "forum_thread_messages",
    "forum_thread_message_versions",
    "private_conversations",
    "private_conversation_participants",
    "private_conversation_messages",
];
pub const STORAGE_FILE_EXTENSION: &str = "txt";
pub const NAME_OF_COOKIE_CONTAINING_SESSION_ID: &str = "session_id";

pub fn get_name_of_cookie_containing_session_id() -> String {
    NAME_OF_COOKIE_CONTAINING_SESSION_ID.to_string()
}

pub trait TextStorable: Sized {
    fn field_count() -> usize;
    fn to_fields(&self) -> Vec<String>;

    fn from_fields(fields: Vec<String>) -> Option<Self>;

    fn file_name() -> &'static str;
}

pub fn read_contents<T: TextStorable>() -> Vec<T> {
    let file_path = get_file_path_gen::<T>();

    let content = fs::read_to_string(file_path.as_str()).unwrap();

    // Remove the ending marker
    let content = content.trim_end_matches(FILE_ENDING_MARKER).trim();

    let mut items = Vec::new();

    // Remove the ending marker
    let content = content.trim_end_matches(FILE_ENDING_MARKER).trim();

    if content.is_empty() {
        return items;
    }
    for entry in content.split_terminator(RECORD_SEPARATOR) {
        let fields: Vec<String> = entry
            .split(FIELD_SEPARATOR)
            .map(|field| {
                let unescaped = storage::unescape(field.to_string());
                unescaped
            })
            .collect();
        if let Some(item) = T::from_fields(fields) {
            items.push(item);
        } else {
            eprintln!("Warning: failed to parse entry {entry}");
        }
    }

    items
}

pub fn write_contents<T: TextStorable>(items: Vec<T>) -> Result<(), std::io::Error> {
    let mut content = String::new();

    for (i, item) in items.iter().enumerate() {
        let escaped_fields: Vec<String> = item
            .to_fields()
            .iter()
            .map(|field| escape(field.to_string()))
            .collect();

        let entry = escaped_fields.join(FIELD_SEPARATOR);
        if i > 0 {
            content.push_str(RECORD_SEPARATOR);
        }
        content.push_str(entry.as_str());
    }

    // add ending marker
    if !content.is_empty() {
        content.push_str(RECORD_SEPARATOR);
    }
    content.push_str(FILE_ENDING_MARKER);

    let file_path = get_file_path_gen::<T>();

    fs::write(file_path, content)
}

pub fn get_file_path_gen<T: TextStorable>() -> String {
    format!(
        "{STORAGE_DIR_NAME}/{}.{STORAGE_FILE_EXTENSION}",
        T::file_name()
    )
}

fn append_before_second_and_last_char(string: String) -> String {
    let mut chars: Vec<char> = string.chars().collect();

    let before_last = String::from("\\");
    let before_second = String::from("\\");
    if chars.len() >= 2 {
        chars.insert(chars.len() - 1, before_last.chars().next().unwrap());

        chars.insert(1, before_second.chars().next().unwrap());
    }
    chars.into_iter().collect()
}
pub fn escape(string: String) -> String {
    let string = string.replace(
        FILE_ENDING_MARKER,
        append_before_second_and_last_char(FILE_ENDING_MARKER.to_string()).as_str(),
    );
    let string = string.replace(
        RECORD_SEPARATOR,
        append_before_second_and_last_char(RECORD_SEPARATOR.to_string()).as_str(),
    );
    let string = string.replace(
        FIELD_SEPARATOR,
        append_before_second_and_last_char(FIELD_SEPARATOR.to_string()).as_str(),
    );

    string
}
pub fn unescape(string: String) -> String {
    let escaped_marker = escape(FILE_ENDING_MARKER.to_string());
    let escaped_record_separator = escape(RECORD_SEPARATOR.to_string());
    let escaped_field_separator = escape(FIELD_SEPARATOR.to_string());

    let string = string.replace(
        escaped_marker.as_str(),
        format!("{FILE_ENDING_MARKER}").as_str(),
    );
    let string = string.replace(
        escaped_record_separator.as_str(),
        format!("{RECORD_SEPARATOR}").as_str(),
    );
    let string = string.replace(
        escaped_field_separator.as_str(),
        format!("{FIELD_SEPARATOR}").as_str(),
    );

    string
}

pub fn initialize_storage_functionality() {
    assert_escapable_file_content_delimeters();
    initialize_dir();

    // should do what initialize_sessions_file does for every file by storing file names in a vec then iterating over them and calling a new function called init_file(name) that basically does what the initialize_sessions_file does but for any file name supplied
    for name in &STORAGE_FILE_NAMES {
        let file_name = format!("{}/{name}.{}", STORAGE_DIR_NAME, STORAGE_FILE_EXTENSION);
        println!("{}", file_name);
        initialize_file(file_name.clone());
    }
}

// elsewhere where I am reading files using the markers/separators, I am split()ting them.
// I think this means that if I want to "escape" the split registering user input as separator  I need to
// make sure the separator is at least two characters long?
fn assert_escapable_file_content_delimeters() {
    //     pub const FILE_ENDING_MARKER
    // pub const RECORD_SEPARATOR
    // pub const FIELD_SEPARATOR
    let min_len = 2;
    assert!(
        (FILE_ENDING_MARKER.len() > min_len)
            && (FIELD_SEPARATOR.len() > min_len)
            && (RECORD_SEPARATOR.len() > min_len),
        "FILE_ENDING_MARKER and FIELD_SEPARATOR and RECORD_SEPARATOR must be longer than 2 chars."
    );
}

fn initialize_file(file_name: String) {
    let file_name = file_name.as_str();
    if let Ok(mut file) = File::open(file_name) {
        // File exists, check if it has the ending marker
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() && !contents.contains(FILE_ENDING_MARKER) {
            let mut file = OpenOptions::new().append(true).open(file_name).unwrap();
            writeln!(file, "{}", FILE_ENDING_MARKER).unwrap();
        }
    } else {
        // Create file with ending marker
        let mut file = File::create(file_name).unwrap();
        writeln!(file, "{}", FILE_ENDING_MARKER).unwrap();
    }
}

fn create_dir_if_not_exists<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    std::fs::create_dir_all(path)
}
fn initialize_dir() {
    println!("Attempting to create directory: {}", STORAGE_DIR_NAME);

    match create_dir_if_not_exists(STORAGE_DIR_NAME) {
        Ok(()) => {
            println!("Directory created successfully or already existed!");
            // You can now safely use the directory
        }
        Err(e) => {
            // It will only error if you lack permissions, or the path
            // points to an existing file instead of a directory.
            eprintln!("Failed to create directory: {}", e);
        }
    }
}

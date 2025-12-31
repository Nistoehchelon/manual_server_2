use std::fs;

/* Used for more convenient usage of resulting executable: in release mode just embeds all files like html and stuff into the
executable. Alternative to doing that: ship the executable together with html_css folder. Possible, but not necessarily
the most convenient alternative.
*/

pub fn read_to_string_on_dev_embed_on_release(file_name: &str) -> String {
    let path = format!("./static_resources/{}", file_name);
    // check if in debug build
    if cfg!(debug_assertions) {
        fs::read_to_string(path.clone()).expect(format!("Could not find file at {}", path).as_str())
    } else {
        // Embed files at compile time in release
        match file_name {
            "footer.html" => include_str!("../../../static_resources/footer.html").to_string(),
            "header.html" => include_str!("../../../static_resources/header.html").to_string(),
            "stylesheet.css" => {
                include_str!("../../../static_resources/stylesheet.css").to_string()
            }
            // Add more files as needed
            _ => {
                panic!("File '{}' not embedded in release build", file_name);
            }
        }
    }
}

pub fn read_on_dev_embed_on_release(file_path: &str) -> Vec<u8> {
    let path = format!("./static_resources/{}", file_path);
    // check if in debug build
    if cfg!(debug_assertions) {
        fs::read(path).unwrap()
    } else {
        // Embed files at compile time in release
        match file_path {
            "favicon.ico" => include_bytes!("../../../static_resources/favicon.ico").to_vec(),
            // Add more files as needed
            _ => {
                panic!("Resource {} not found ", path);
            }
        }
    }
}

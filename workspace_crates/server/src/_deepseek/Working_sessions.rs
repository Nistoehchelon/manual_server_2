use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write, Read};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    // Initialize sessions.txt with ending marker if it doesn't exist
    initialize_sessions_file();
    
    let listener = TcpListener::bind("127.0.0.1:8080").expect("Failed to bind to address");
    println!("Server listening on http://127.0.0.1:8080");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| {
                    handle_client(stream);
                });
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}

fn initialize_sessions_file() {
    if let Ok(mut file) = File::open("sessions.txt") {
        // File exists, check if it has the ending marker
        let mut contents = String::new();
        if file.read_to_string(&mut contents).is_ok() {
            if !contents.contains("]]") {
                let mut file = OpenOptions::new().append(true).open("sessions.txt").unwrap();
                writeln!(file, "]]").unwrap();
            }
        }
    } else {
        // Create file with ending marker
        let mut file = File::create("sessions.txt").unwrap();
        writeln!(file, "]]").unwrap();
    }
}

fn handle_client(mut stream: TcpStream) {
    let mut reader = BufReader::new(&stream);
    
    // Read the request line
    let mut request_line = String::new();
    if reader.read_line(&mut request_line).is_err() {
        return;
    }
    
    let parts: Vec<&str> = request_line.split_whitespace().collect();
    if parts.len() < 2 {
        return;
    }
    
    let method = parts[0];
    let path_with_query = parts[1];
    
    // Read headers
    let mut headers = HashMap::new();
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).is_err() || line == "\r\n" || line == "\n" {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.insert(key.trim().to_lowercase(), value.trim().to_string());
        }
    }
    
    // Read body for POST requests
    let mut body = String::new();
    if method == "POST" {
        // Check content-length header
        if let Some(content_length_str) = headers.get("content-length") {
            if let Ok(content_length) = content_length_str.parse::<usize>() {
                let mut body_bytes = vec![0; content_length];
                if reader.read_exact(&mut body_bytes).is_ok() {
                    body = String::from_utf8_lossy(&body_bytes).to_string();
                }
            }
        }
    }
    
    // Parse cookies
    let cookies = parse_cookies(&headers);
    
    // Parse path and query
    let (path, _query) = parse_path_and_query(path_with_query);
    
    // Route handling
    let response = match (method, path.as_str()) {
        ("GET", "/username") => handle_get_username(&cookies),
        ("POST", "/username") => handle_post_username(&body, &cookies),
        ("GET", "/dashboard") => handle_get_dashboard(&cookies),
        ("GET", "/logout") => handle_get_logout(&cookies),
        _ => handle_default(),
    };
    
    stream.write_all(response.as_bytes()).ok();
}

fn parse_path_and_query(path_with_query: &str) -> (String, HashMap<String, String>) {
    let parts: Vec<&str> = path_with_query.split('?').collect();
    let path = parts[0].to_string();
    let mut query = HashMap::new();
    
    if parts.len() > 1 {
        for pair in parts[1].split('&') {
            let kv: Vec<&str> = pair.split('=').collect();
            if kv.len() == 2 {
                query.insert(kv[0].to_string(), kv[1].to_string());
            }
        }
    }
    
    (path, query)
}

fn parse_cookies(headers: &HashMap<String, String>) -> HashMap<String, String> {
    let mut cookies = HashMap::new();
    if let Some(cookie_header) = headers.get("cookie") {
        for cookie in cookie_header.split(';') {
            let parts: Vec<&str> = cookie.trim().split('=').collect();
            if parts.len() == 2 {
                cookies.insert(parts[0].to_string(), parts[1].to_string());
            }
        }
    }
    cookies
}

fn handle_get_username(cookies: &HashMap<String, String>) -> String {
    // Check if session exists in cookies
    if let Some(sess_id) = cookies.get("sessid") {
        // Check sessions.txt for this session
        if let Ok(sessions) = read_sessions() {
            if let Some(session) = sessions.get(sess_id) {
                if session.logged_in {
                    // User is logged in, redirect to dashboard
                    return format!(
                        "HTTP/1.1 302 Found\r\n\
                         Location: /dashboard\r\n\
                         Content-Type: text/html\r\n\r\n"
                    );
                }
            }
        }
    }
    
    // Show username form
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
         <!DOCTYPE html>
         <html>
         <head>
             <title>Enter Username</title>
             <style>
                 body {{ font-family: Arial, sans-serif; max-width: 500px; margin: 50px auto; padding: 20px; }}
                 form {{ display: flex; flex-direction: column; gap: 10px; }}
                 input[type='text'] {{ padding: 10px; font-size: 16px; }}
                 input[type='submit'] {{ padding: 10px 20px; background: #007bff; color: white; border: none; cursor: pointer; }}
             </style>
         </head>
         <body>
             <h1>Enter Username</h1>
             <form method='POST' action='/username'>
                 <input type='text' name='username' placeholder='Enter your username' required>
                 <input type='submit' value='Submit'>
             </form>
         </body>
         </html>"
    )
}

fn handle_post_username(body: &str, cookies: &HashMap<String, String>) -> String {
    // Parse username from form data
    let mut username = String::new();
    for pair in body.split('&') {
        let kv: Vec<&str> = pair.split('=').collect();
        if kv.len() == 2 && kv[0] == "username" {
            username = kv[1].to_string();
            break;
        }
    }
    
    if username.is_empty() {
        return format!(
            "HTTP/1.1 400 Bad Request\r\n\
             Content-Type: text/html\r\n\r\n\
             <h1>Username is required</h1>"
        );
    }
    
    // Check if we already have a session
    let sess_id = if let Some(existing_id) = cookies.get("sessid") {
        existing_id.clone()
    } else {
        // Generate new session ID
        generate_session_id()
    };
    
    // Update or create session
    let mut sessions = read_sessions().unwrap_or_default();
    sessions.insert(sess_id.clone(), Session {
        username: username.clone(),
        logged_in: true,
    });
    
    // Write sessions to file
    if write_sessions(&sessions).is_err() {
        return format!(
            "HTTP/1.1 500 Internal Server Error\r\n\
             Content-Type: text/html\r\n\r\n\
             <h1>Server error</h1>"
        );
    }
    
    // Redirect to dashboard with session cookie
    format!(
        "HTTP/1.1 302 Found\r\n\
         Set-Cookie: sessid={}; Path=/\r\n\
         Location: /dashboard\r\n\
         Content-Type: text/html\r\n\r\n",
        sess_id)
}

fn handle_get_dashboard(cookies: &HashMap<String, String>) -> String {
    // Check if session exists
    let (sess_id, sessions) = match cookies.get("sessid") {
        Some(id) => {
            if let Ok(sessions) = read_sessions() {
                (id.clone(), sessions)
            } else {
                // Can't read sessions, redirect to username
                return redirect_to_username();
            }
        }
        None => {
            // No session cookie, create one and redirect to username
            let new_id = generate_session_id();
            let mut sessions = read_sessions().unwrap_or_default();
            sessions.insert(new_id.clone(), Session {
                username: String::new(),
                logged_in: false,
            });
            
            if write_sessions(&sessions).is_err() {
                return redirect_to_username();
            }
            
            return format!(
                "HTTP/1.1 302 Found\r\n\
                 Set-Cookie: sessid={}; Path=/\r\n\
                 Location: /username\r\n\
                 Content-Type: text/html\r\n\r\n",
                new_id
            );
        }
    };
    
    // Check if session exists and user is logged in
    if let Some(session) = sessions.get(&sess_id) {
        if session.logged_in && !session.username.is_empty() {
            // Show dashboard
            return format!(
                "HTTP/1.1 200 OK\r\n\
                 Content-Type: text/html\r\n\r\n\
                 <!DOCTYPE html>
                 <html>
                 <head>
                     <title>Dashboard</title>
                     <style>
                         body {{ font-family: Arial, sans-serif; max-width: 500px; margin: 50px auto; padding: 20px; }}
                         a {{ color: #007bff; text-decoration: none; }}
                         a:hover {{ text-decoration: underline; }}
                     </style>
                 </head>
                 <body>
                     <h1>Hello, {}!</h1>
                     <p>Welcome to your dashboard.</p>
                     <p><a href='/logout'>Logout</a></p>
                 </body>
                 </html>",
                session.username
            );
        }
    }
    
    // Session doesn't exist or user not logged in
    redirect_to_username()
}

fn handle_get_logout(cookies: &HashMap<String, String>) -> String {
    if let Some(sess_id) = cookies.get("sessid") {
        let mut sessions = read_sessions().unwrap_or_default();
        if let Some(session) = sessions.get_mut(sess_id) {
            session.logged_in = false;
            if write_sessions(&sessions).is_ok() {
                // Clear the session cookie and redirect to username
                return format!(
                    "HTTP/1.1 302 Found\r\n\
                     Set-Cookie: sessid=; Path=/; Max-Age=0\r\n\
                     Location: /username\r\n\
                     Content-Type: text/html\r\n\r\n"
                );
            }
        }
    }
    
    redirect_to_username()
}

fn handle_default() -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html\r\n\r\n\
         <!DOCTYPE html>
         <html>
         <head>
             <title>Session Server</title>
             <style>
                 body {{ font-family: Arial, sans-serif; max-width: 600px; margin: 50px auto; padding: 20px; }}
                 ul {{ line-height: 1.6; }}
                 a {{ color: #007bff; text-decoration: none; }}
                 a:hover {{ text-decoration: underline; }}
             </style>
         </head>
         <body>
             <h1>Session Server</h1>
             <p>Available endpoints:</p>
             <ul>
                 <li><a href='/username'>/username</a> - Enter username to login</li>
                 <li><a href='/dashboard'>/dashboard</a> - Dashboard (requires login)</li>
                 <li><a href='/logout'>/logout</a> - Logout</li>
             </ul>
         </body>
         </html>"
    )
}

fn redirect_to_username() -> String {
    format!(
        "HTTP/1.1 302 Found\r\n\
         Location: /username\r\n\
         Content-Type: text/html\r\n\r\n"
    )
}

fn generate_session_id() -> String {
    // Simple session ID generator using timestamp and random number
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_micros();
    
    // Simple pseudo-random string
    format!("sess_{:x}", timestamp)
}

#[derive(Debug, Clone)]
struct Session {
    username: String,
    logged_in: bool,
}

fn read_sessions() -> Result<HashMap<String, Session>, std::io::Error> {
    let mut sessions = HashMap::new();
    
    let content = std::fs::read_to_string("sessions.txt")?;
    
    // Remove the ending marker
    let content = content.trim_end_matches("]]").trim();
    
    // Split by >>
    if !content.is_empty() {
        for entry in content.split(">>") {
            let parts: Vec<&str> = entry.split("||").collect();
            if parts.len() == 3 {
                let sess_id = parts[0].to_string();
                let username = parts[1].to_string();
                let logged_in = parts[2] == "true";
                
                sessions.insert(sess_id, Session {
                    username,
                    logged_in,
                });
            }
        }
    }
    
    Ok(sessions)
}

fn write_sessions(sessions: &HashMap<String, Session>) -> Result<(), std::io::Error> {
    let mut content = String::new();
    
    for (sess_id, session) in sessions {
        let entry = format!("{}||{}||{}", sess_id, session.username, session.logged_in);
        if !content.is_empty() {
            content.push_str(">>");
        }
        content.push_str(&entry);
    }
    
    // Add the ending marker
    content.push_str(">>]]");
    
    std::fs::write("sessions.txt", content)
}
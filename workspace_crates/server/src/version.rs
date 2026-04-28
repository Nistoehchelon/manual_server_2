use git_version::git_version;

pub const VERSION: &str = git_version!(args = ["--tags", "--abbrev=0" ]); /*  increase when I want to compare which one I am running on prod 
just --tags shows some sort of hash at the end.
--abbrev being set to 0 is necessary to return only the tag.
*/
pub fn version_cli() -> bool /* <-- whether I wanna continue doing whatever I was doing after calling handle_args(). To be more specific, in my current use case I will basically use this to determine if I called -v or just started executable normally.  */ {
    let mut args = std::env::args().skip(1);
    match args.next() {
        None => {
            true
        },
        Some(arg) if arg == "-v" || arg == "--version" => {
            println!("App version: {}", VERSION);
            false
        },
        Some(other) => {
            eprintln!("Unknown argument: {}", other);
            eprintln!("Usage: app [-v | --version]");
            false
        }
    }
} 
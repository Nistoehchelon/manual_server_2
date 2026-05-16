#[derive(Debug, Default)]
pub struct Args {
    pub version: bool,
}

pub fn parse_args() -> Result<Args, String> {
    let mut args = Args::default();

    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-v" | "--version" => args.version = true,
            other => {
                return Err(format!(
                    "Unknown argument: {other}\nUsage: app [-v | --version]"
                ));
            }
        }
    }

    Ok(args)
}

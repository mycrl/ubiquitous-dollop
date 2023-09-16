use clap::Parser;

#[derive(Parser)]
#[command(
    about = env!("CARGO_PKG_DESCRIPTION"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
)]
pub struct Config {
    #[arg(env, long, default_value = "webview://index.html")]
    pub url: String,
}

impl Config {
    pub fn new() -> Self {
        Config::parse()
    }
}

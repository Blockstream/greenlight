use clap::Parser;
use gl_cli::{run, Cli};

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match run(cli).await {
        Ok(()) => (),
        Err(e) => {
            println!("{}", e);
        }
    }
}

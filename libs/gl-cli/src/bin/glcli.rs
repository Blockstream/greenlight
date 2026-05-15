use clap::Parser;
use gl_cli::{run, Cli};
use serde_json::json;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let json_print = cli.json;

    match run(cli).await {
        Ok(()) => (),
        Err(e) => {
            if json_print {
                let j = json!({"error": e.to_string()});
                println!("{}", serde_json::to_string_pretty(&j).unwrap());
            } else {
                println!("{}", e);
            }
        }
    }
}

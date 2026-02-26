use clap::Parser;

fn main() {
    let cli = gl_sdk_cli::Cli::parse();
    if let Err(e) = gl_sdk_cli::run(cli) {
        e.print_and_exit();
    }
}

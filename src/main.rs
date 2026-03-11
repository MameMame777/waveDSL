use clap::Parser;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wavedsl", about = "WaveDSL compiler — converts WaveDSL to WaveDrom JSON")]
struct Cli {
    /// Input file (reads from stdin if omitted)
    input: Option<PathBuf>,

    /// Output file (writes to stdout if omitted; pretty-prints automatically)
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    let input = match &cli.input {
        Some(path) => std::fs::read_to_string(path).unwrap_or_else(|e| {
            eprintln!("error: cannot read '{}': {}", path.display(), e);
            std::process::exit(1);
        }),
        None => {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf).unwrap_or_else(|e| {
                eprintln!("error: cannot read stdin: {}", e);
                std::process::exit(1);
            });
            buf
        }
    };

    match wavedsl::compile(&input) {
        Ok(json) => {
            let text = serde_json::to_string_pretty(&json).unwrap();
            match &cli.output {
                Some(path) => {
                    std::fs::write(path, text.as_bytes()).unwrap_or_else(|e| {
                        eprintln!("error: cannot write '{}': {}", path.display(), e);
                        std::process::exit(1);
                    });
                }
                None => println!("{}", text),
            }
        }
        Err(errors) => {
            for err in &errors {
                eprintln!("error: {}", err);
            }
            std::process::exit(1);
        }
    }
}

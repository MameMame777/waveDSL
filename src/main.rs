use clap::Parser;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "wavedsl", about = "WaveDSL compiler — converts WaveDSL to WaveDrom JSON (and optionally SystemVerilog assertions)")]
struct Cli {
    /// Input file (reads from stdin if omitted)
    input: Option<PathBuf>,

    /// Output file for WaveDrom JSON (writes to stdout if omitted)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output file for SystemVerilog assertions
    /// (auto-named <input>.sv for file inputs; requires --sv when reading stdin)
    #[arg(long)]
    sv: Option<PathBuf>,

    /// Suppress SystemVerilog assertion output entirely
    #[arg(long)]
    no_sv: bool,
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

    match wavedsl::compile_full(&input, cli.input.as_deref()) {
        Ok((json, sv)) => {
            // --- JSON output ---
            let json_text = serde_json::to_string_pretty(&json).unwrap();
            match &cli.output {
                Some(path) => {
                    std::fs::write(path, json_text.as_bytes()).unwrap_or_else(|e| {
                        eprintln!("error: cannot write '{}': {}", path.display(), e);
                        std::process::exit(1);
                    });
                }
                None => println!("{}", json_text),
            }

            // --- SV output ---
            if !cli.no_sv {
                if let Some(sv_text) = sv {
                    let sv_path = cli.sv.clone().or_else(|| {
                        cli.input.as_ref().map(|p| p.with_extension("sv"))
                    });
                    match sv_path {
                        Some(path) => {
                            std::fs::write(&path, sv_text.as_bytes()).unwrap_or_else(|e| {
                                eprintln!("error: cannot write '{}': {}", path.display(), e);
                                std::process::exit(1);
                            });
                            eprintln!("info: SystemVerilog assertions written to '{}'", path.display());
                        }
                        None => {
                            // stdin mode without --sv: nothing to do silently
                        }
                    }
                }
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


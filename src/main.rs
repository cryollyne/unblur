use std::fmt::Display;

use clap::Parser;

#[derive(clap::ValueEnum, Debug, Clone, Copy)]
enum Kernel {
    Box,
    Gaussian,
}

impl Display for Kernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("{self:?}").to_lowercase())
    }
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = Kernel::Gaussian)]
    kernel: Kernel,
    #[arg(long)]
    kernel_file: Option<String>,

    #[arg(short, long, default_value_t = 1f32)]
    size: f32,

    image: String,
}

fn main() {
    let args = Args::parse();
}

#[derive(clap::Parser, Debug)]
#[command(author, about)]
pub struct Cli {
    #[clap(short, long)]
    pub addr: Option<String>,

    #[clap(short, long)]
    pub bootstrap: Vec<String>,
}

use env_logger;

mod utils;

pub use utils::Block;
pub use utils::Blockchain;
pub use utils::Cli;
pub use utils::Result;

pub const BLOCKCHAIN_DATA_PATH: &str = "data/blocks";

fn main() -> Result<()> {
  std::env::set_var("RUST_LOG", "info"); /* disable if you dont want logs by default */
  env_logger::init();
  let mut cli = Cli::new()?;

  cli.run()?;
  Ok(())
}

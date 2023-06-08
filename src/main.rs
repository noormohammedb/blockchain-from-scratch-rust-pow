mod utils;

pub use utils::Block;
pub use utils::Blockchain;
pub use utils::Cli;
pub use utils::Result;

pub const BLOCKCHAIN_DATA_PATH: &str = "data/blocks";

fn main() -> Result<()> {
  let mut cli = Cli::new()?;

  cli.run()?;
  Ok(())
}

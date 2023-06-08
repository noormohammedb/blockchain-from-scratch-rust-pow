use clap::arg;
use clap::Command;

use crate::Blockchain;
use crate::Result;

pub struct Cli {
  bc: Blockchain,
}

impl Cli {
  pub fn new() -> Result<Cli> {
    Ok(Cli {
      bc: Blockchain::new()?,
    })
  }
  pub fn run(&mut self) -> Result<()> {
    let matches = Command::new("rust-blockchain")
      .version("0.1")
      .author("Noor")
      .about("Blockchain written in rust: a simple pow blockchain written in rust from scratch")
      .subcommand(Command::new("printchain").about("print all the chain blocks"))
      .subcommand(
        Command::new("addblock")
          .about("add a block in the blockchain")
          .arg(arg!(<DATA>" 'the blockchain data' ")),
      )
      .get_matches();

    if let Some(matches) = matches.subcommand_matches("addblock") {
      if let Some(c) = matches.get_one::<String>("DATA") {
        self.bc.add_block(c.clone())?;
      }
    } else if let Some(_) = matches.subcommand_matches("printchain") {
      self.print_chain();
    } else {
      println!("command not found");
    }

    Ok(())
  }

  fn addblock(&mut self, data: String) -> Result<()> {
    self.bc.add_block(data)
  }

  fn print_chain(&mut self) {
    for b in &mut self.bc.iter().rev() {
      println!("block: {:#?}", b);
    }
  }
}

use crate::{block::Block, errors::Result, BLOCKCHAIN_DATA_PATH};

#[derive(Debug, Clone)]
pub struct Blockchain {
  blocks: Vec<Block>,
  current_hash: String,
  db: sled::Db,
}

impl Blockchain {
  pub fn new() -> Result<Blockchain> {
    let db = sled::open(BLOCKCHAIN_DATA_PATH)?;
    match db.get("LAST")? {
      Some(hash) => {
        let last_hash = String::from_utf8(hash.to_vec())?;
        Ok(Blockchain {
          blocks: Vec::new(),
          current_hash: last_hash,
          db,
        })
      }
      None => {
        let block = Block::new_genesis_block();
        let current_hash = block.get_hash();
        db.insert(current_hash.clone(), bincode::serialize(&block)?);
        db.insert("LAST", bincode::serialize(&block)?)?;
        let bc = Blockchain {
          blocks: vec![block],
          current_hash: current_hash,
          db,
        };
        bc.db.flush()?;
        Ok(bc)
      }
    }
  }

  pub fn add_block(&mut self, data: String) -> Result<()> {
    let last_block_blob = self.db.get("LAST")?.unwrap();
    let last_block = bincode::deserialize::<Block>(&last_block_blob)?;
    let new_block = Block::new_block(data, last_block.get_hash(), last_block.get_height() + 1)?;
    let current_hash = new_block.get_hash();
    self
      .db
      .insert(current_hash.clone(), bincode::serialize(&new_block)?);
    self.db.insert("LAST", bincode::serialize(&new_block)?);
    self.blocks.push(new_block);
    self.current_hash = current_hash;
    Ok(())
  }

  pub fn get_blocks(&self) -> Vec<Block> {
    self.blocks.clone()
  }

  pub fn get_data(&self) -> (&Vec<Block>, &String) {
    (&self.blocks, &self.current_hash)
  }

    }
}

use crate::{block::Block, errors::Result, BLOCKCHAIN_DATA_PATH};

#[derive(Debug, Clone)]
pub struct Blockchain {
  blocks: Vec<Block>,
  current_hash: String,
  db: sled::Db,
}
pub struct BlockchainIter {
  current_hash: String,
  bc: Blockchain,
}

impl Blockchain {
  pub fn new() -> Result<Blockchain> {
    let db = sled::open(BLOCKCHAIN_DATA_PATH)?;
    match db.get("LAST")? {
      Some(last_block_blob) => {
        let last_block = bincode::deserialize::<Block>(&last_block_blob)?;
        let mut chain_with_db_ref = Blockchain {
          blocks: Vec::new(),
          current_hash: last_block.get_hash(),
          db: db.clone(),
        };

        let mut blocks = vec![];
        for block in chain_with_db_ref.iter() {
          blocks.insert(0, block);
        }

        Ok(Blockchain {
          blocks: blocks,
          current_hash: last_block.get_hash(),
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

  pub fn iter(&self) -> BlockchainIter {
    BlockchainIter {
      current_hash: self.current_hash.clone(),
      bc: self.clone(),
    }
  }
}

impl Iterator for BlockchainIter {
  type Item = Block;
  fn next(&mut self) -> Option<Self::Item> {
    if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
      match encoded_block {
        Some(block_byte) => {
          if let Ok(block) = bincode::deserialize::<Block>(&block_byte) {
            self.current_hash = block.get_prev_hash();
            return Some(block);
          } else {
            return None;
          }
        }
        None => None,
      }
    } else {
      None
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_add_block() {
    let mut b = Blockchain::new().unwrap();
    b.add_block("data 1".to_string());
    b.add_block("data 2".to_string());
    b.add_block("data 3".to_string());

    dbg!(b.get_data());

    // for block in b.iter() {
    //   dbg!(block);
    // }
  }
}

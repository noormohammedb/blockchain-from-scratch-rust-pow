use crate::{Block, Result, BLOCKCHAIN_DATA_PATH};

#[derive(Debug, Clone)]
pub struct Blockchain {
  blocks: Vec<Block>,
  current_hash: String,
  db: sled::Db,
}
pub struct BlockchainIter {
  current_hash: String,
  height: usize,
  bc: Blockchain,
}

impl Blockchain {
  pub fn new() -> Result<Blockchain> {
    let db = sled::open(BLOCKCHAIN_DATA_PATH)?;
    match db.get("LAST")? {
      Some(last_block_hash_blob) => {
        let last_block_hash = bincode::deserialize::<String>(&last_block_hash_blob)?;
        let last_block_blob = db.get(last_block_hash)?.unwrap();
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
        db.insert("LAST", bincode::serialize(&current_hash)?)?;
        db.insert("FIRST", bincode::serialize(&current_hash)?)?; // for genesis block
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
    let last_block_hash_blob = self.db.get("LAST")?.unwrap();
    let last_block_hash = bincode::deserialize::<String>(&last_block_hash_blob)?;
    let last_block_encoded = self.db.get(last_block_hash)?.unwrap();
    let last_block = bincode::deserialize::<Block>(&last_block_encoded)?;

    let last_height = last_block.get_height();
    let new_height = last_height + 1;

    let new_block = Block::new_block(data, last_block.get_hash(), new_height)?;
    let current_hash = new_block.get_hash();
    self
      .db
      .insert(current_hash.clone(), bincode::serialize(&new_block)?);
    self.db.insert("LAST", bincode::serialize(&current_hash)?);

    // to track from 0 - last block
    self
      .db
      .insert(last_height.to_string(), bincode::serialize(&current_hash)?);
    self.blocks.push(new_block);
    self.current_hash = current_hash;
    println!("Block added with height: {}", new_height);
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
      height: Default::default(),
    }
  }
}

impl Blockchain {
  fn get_block_with_prev_height(&self, height: usize) -> Option<Block> {
    if let Ok(encoded_block_hash_option) = self.db.get(height.to_string()) {
      match encoded_block_hash_option {
        Some(encoded_block_hash) => {
          if let Ok(block_hash) = bincode::deserialize::<String>(&encoded_block_hash) {
            if block_hash.len() < 5 {
              let block = self.get_last_block();
              return block;
            }
            if let Ok(encoded_block_option) = self.db.get(block_hash) {
              match encoded_block_option {
                Some(encoded_block) => {
                  if let Ok(block) = bincode::deserialize::<Block>(&encoded_block) {
                    Some(block)
                  } else {
                    None
                  }
                }
                None => self.get_last_block(),
              }
            } else {
              None
            }
          } else {
            None
          }
        }
        None => None,
      }
    } else {
      None
    }
  }

  fn get_genesis_block(&self) -> Option<Block> {
    if let Ok(encoded_genesis_hash_option) = self.db.get("FIRST") {
      match encoded_genesis_hash_option {
        Some(encoded_genesis_hash) => {
          if let Ok(genesis_hash) = bincode::deserialize::<String>(&encoded_genesis_hash) {
            self.get_block_by_hash(genesis_hash)
          } else {
            None
          }
        }
        None => None,
      }
    } else {
      None
    }
  }

  fn get_last_block(&self) -> Option<Block> {
    if let Ok(encoded_last_block_hash_option) = self.db.get("LAST") {
      match encoded_last_block_hash_option {
        Some(encoded_last_block_hash) => {
          if let Ok(last_block_hash) = bincode::deserialize::<String>(&encoded_last_block_hash) {
            if let Ok(encoded_block_option) = self.db.get(last_block_hash) {
              match encoded_block_option {
                Some(encoded_block) => {
                  if let Ok(block) = bincode::deserialize::<Block>(&encoded_block) {
                    Some(block)
                  } else {
                    None
                  }
                }
                None => None,
              }
            } else {
              None
            }
          } else {
            None
          }
        }
        None => None,
      }
      //
    } else {
      None
    }
  }
  fn get_block_by_hash(&self, hash: String) -> Option<Block> {
    if let Ok(encoded_block_option) = self.db.get(hash) {
      match encoded_block_option {
        Some(encoded_block) => {
          if let Ok(block) = bincode::deserialize::<Block>(&encoded_block) {
            Some(block)
          } else {
            None
          }
        }
        None => None,
      }
      //
    } else {
      None
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

impl DoubleEndedIterator for BlockchainIter {
  fn next_back(&mut self) -> Option<Self::Item> {
    if self.height == 0 {
      self.height += 1;
      return self.bc.get_genesis_block();
    }
    let current_block = self.bc.get_block_with_prev_height(self.height - 1);
    self.height += 1;
    current_block
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

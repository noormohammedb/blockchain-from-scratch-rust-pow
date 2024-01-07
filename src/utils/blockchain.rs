use sled::{self, Db};
use std::collections::HashMap;

use log::info;

use crate::{utils::Transaction, Block, Result, BLOCKCHAIN_DATA_PATH};

use super::transactions::TXOutput;

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
  pub fn create_blockchain(db: Db, address: String) -> Result<Blockchain> {
    info!("Creating new block database");
    // let db = sled::open(BLOCKCHAIN_DATA_PATH)?;
    let cbtx = Transaction::new_coinbase(address, String::from("Genesis transaction"))?;
    let genesis: Block = Block::new_genesis_block(cbtx);
    let genesis_hash = genesis.get_hash();
    db.insert(&genesis_hash, bincode::serialize(&genesis)?)?;
    db.insert("LAST", bincode::serialize(&genesis_hash)?)?;
    db.insert("FIRST", bincode::serialize(&genesis_hash)?)?; // for genesis block
    let bc = Blockchain {
      current_hash: genesis_hash,
      blocks: vec![genesis],
      db,
    };

    bc.db.flush()?;

    Ok(bc)
  }
  pub fn new() -> Result<Blockchain> {
    info!("open blockchain");

    let mut db = sled::open(BLOCKCHAIN_DATA_PATH)?;

    if !db.was_recovered() {
      println!("creating new blockchain with genesis block");
      db = Self::create_blockchain(db.clone(), "0".to_owned())?.db;
    }
    let hash = db.get("LAST")?;
    let last_hash = String::from_utf8(hash.unwrap_or_default().to_vec())?;
    let mut blockchain_with_db_ref = Blockchain {
      blocks: vec![],
      current_hash: last_hash,
      db,
    };

    let mut blocks = vec![];
    for block in blockchain_with_db_ref.iter() {
      blocks.insert(0, block);
    }
    blockchain_with_db_ref.blocks = blocks;
    Ok(blockchain_with_db_ref)
  }
  /*
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
   */

  pub fn add_block(&mut self, data: Vec<Transaction>) -> Result<()> {
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
impl Blockchain {
  fn find_unspent_transactions(&self, address: &str) -> Vec<Transaction> {
    let mut spend_TXOs: HashMap<String, Vec<i32>> = HashMap::new();
    let mut unspend_TXOs: Vec<Transaction> = Vec::new();

    for block in self.iter() {
      for tx in block.get_transaction() {
        for index in 0..tx.vout.len() {
          if let Some(ids) = spend_TXOs.get(&tx.id) {
            if ids.contains(&(index as i32)) {
              continue;
            }
          }

          if tx.vout[index].can_be_unlock_with(address) {
            unspend_TXOs.push(tx.to_owned())
          }
        }

        if !tx.is_coinbase() {
          for i in &tx.vin {
            if i.can_be_unlock_with(address) {
              match spend_TXOs.get_mut(&i.txid) {
                Some(v) => {
                  v.push(i.vout);
                }
                None => {
                  spend_TXOs.insert(i.txid.clone(), vec![i.vout]);
                }
              }
            }
          }
        }
      }
    }
    unspend_TXOs
  }

  pub fn find_UTXO(&self, address: &str) -> Vec<TXOutput> {
    let mut utxos = Vec::<TXOutput>::new();
    let unspend_TXOs = self.find_unspent_transactions(address);

    for tx in unspend_TXOs {
      for out in &tx.vout {
        if out.can_be_unlock_with(&address) {
          utxos.push(out.clone());
        }
      }
    }
    utxos
  }
  // pub fn find_spendable_outputs(
  //   &self,
  //   address: &str,
  //   amount: i32,
  // ) -> (i32, HashMap<String, Vec<i32>>) {
  //   let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
  //   let mut accumulated = 0;
  //   let unspend_TXOs = self.find_unspent_transactions(address);
  //   //
  // }
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
    // b.add_block("data 1".to_string());
    // b.add_block("data 2".to_string());
    // b.add_block("data 3".to_string());

    b.add_block(vec![]);
    b.add_block(vec![]);
    b.add_block(vec![]);

    dbg!(b.get_data());

    // for block in b.iter() {
    //   dbg!(block);
    // }
  }
}

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use log::info;
use std::time::SystemTime;

pub type Result<T> = std::result::Result<T, failure::Error>;

const TARGET_HEXT: usize = 4;
struct Block {
  timestamp: u128,
  transactions: String,
  prev_block_hash: String,
  hash: String,
  height: usize,
  nonce: i32,
}

pub struct Blockchain {
  blocks: Vec<Block>,
}

impl Block {
  pub fn new_block(data: String, prev_block_hash: String, height: usize) -> Result<Block> {
    let timestamp = SystemTime::now()
      .duration_since(SystemTime::UNIX_EPOCH)?
      .as_millis();

    let mut block = Block {
      timestamp,
      transactions: data,
      prev_block_hash,
      hash: String::new(),
      height,
      nonce: 0,
    };
    block.run_proof_of_work()?;
    Ok(block)
  }

  fn run_proof_of_work(&mut self) -> Result<()> {
    info!("Mining the block");
    while !self.validate()? {
      self.nonce += 1;
    }
    let data = self.prepare_hash_data()?;
    let mut hasher = Sha256::new();
    hasher.input(&data[..]);
    self.hash = hasher.result_str();
    Ok(())
  }

  fn validate(&self) -> Result<bool> {
    let data = self.prepare_hash_data()?;

    let mut hasher = Sha256::new();
    hasher.input(&data[..]);
    let mut vec1 = vec![];
    vec1.resize(TARGET_HEXT, '0' as u8);
    println!("{:?}", vec1);

    Ok(&hasher.result_str()[0..TARGET_HEXT] == String::from_utf8(vec1)?)
  }

  fn prepare_hash_data(&self) -> Result<Vec<u8>> {
    let content = (
      self.prev_block_hash.clone(),
      self.transactions.clone(),
      self.timestamp,
      TARGET_HEXT,
      self.nonce,
    );
    let bytes = bincode::serialize(&content)?;
    Ok(bytes)
  }
}

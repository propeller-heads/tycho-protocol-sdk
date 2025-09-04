use anyhow::{Ok, Result};
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    Abigen::new("Factory", "abi/CLFactory.json")?
        .generate()?
        .write_to_file("src/abi/clfactory.rs")?;
    Abigen::new("Pool", "abi/CLPool.json")?
        .generate()?
        .write_to_file("src/abi/clpool.rs")?;
    Ok(())
}

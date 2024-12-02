use anyhow::{Ok, Result};
use regex::Regex;
use std::fs;
use substreams_ethereum::Abigen;

fn main() -> Result<(), anyhow::Error> {
    let file_names = ["abi/pool_contract.abi.json", "abi/erc20.abi.json"];
    let file_output_names = ["src/abi/pool_contract.rs", "src/abi/erc20.rs"];

    let regex = Regex::new(r#"("\w+"\s?:\s?")_(\w+")"#).unwrap();
    for (i, f) in file_names.into_iter().enumerate() {
        let contents = fs::read_to_string(f).expect("Should have been able to read the file");

        // sanitize fields and attributes starting with an underscore
        let sanitized_abi_file = regex.replace_all(contents.as_str(), "${1}u_${2}");

        Abigen::from_bytes("Contract", sanitized_abi_file.as_bytes())?
            .generate()?
            .write_to_file(file_output_names[i])?;
    }

    Ok(())
}

use anyhow::Result;
use std::{fs, io::Write};
use substreams_ethereum::Abigen;

fn main() -> Result<()> {
    let abi_folder = "abi";
    let output_folder = "src/abi";

    let files = fs::read_dir(abi_folder)?;
    let mut mod_rs_content = String::new();

    for file in files {
        let file = file?;
        let file_name = file.file_name();
        let file_name = file_name.to_string_lossy();

        if !file_name.ends_with(".json") {
            continue;
        }

        let contract_name = file_name.split('.').next().unwrap();

        let input_path = format!("{}/{}", abi_folder, file_name);
        let output_path = format!("{}/{}.rs", output_folder, contract_name);

        mod_rs_content.push_str(&format!("pub mod {};\n", contract_name));

        if std::path::Path::new(&output_path).exists() {
            continue;
        }

        Abigen::new(contract_name, &input_path)?
            .generate()?
            .write_to_file(&output_path)?;
    }

    let mod_rs_path = format!("{}/mod.rs", output_folder);
    let mut mod_rs_file = fs::File::create(mod_rs_path)?;

    mod_rs_file.write_all(mod_rs_content.as_bytes())?;

    Ok(())
}

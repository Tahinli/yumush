use std::{
    collections::{HashMap, VecDeque},
    fs::File,
    io::Read,
    path::Path,
};

use crate::error::{Error, file_operation};

#[derive(Debug, Clone)]
pub struct TOML {
    pub header: String,
    pub fields: HashMap<String, String>,
}

impl TOML {
    pub fn naive_toml_parser(file_location: impl AsRef<Path>) -> Result<TOML, Error> {
        let mut toml_file = File::open(file_location)
            .map_err(|error_value| file_operation::Error::FileOpen(error_value.to_string()))?;
        let mut toml_ingredients = String::default();
        toml_file
            .read_to_string(&mut toml_ingredients)
            .map_err(|_| file_operation::Error::Read)?;
        let mut toml_ingredients = toml_ingredients.lines().collect::<VecDeque<&str>>();

        let header = toml_ingredients
            .pop_front()
            .ok_or(file_operation::Error::Empty)?
            .replace('[', "")
            .replace(']', "")
            .trim_end()
            .to_string();

        let mut fields = HashMap::new();

        toml_ingredients
            .iter()
            .try_for_each(|ingredient| -> Result<(), Error> {
                let key_and_value = ingredient
                    .split_once('=')
                    .ok_or(file_operation::Error::Split)?;

                let key = key_and_value.0.replace('"', "").trim().to_string();
                let value = key_and_value.1.replace('"', "").trim().to_string();

                fields.insert(key, value);
                Ok(())
            })?;

        Ok(TOML { header, fields })
    }
}

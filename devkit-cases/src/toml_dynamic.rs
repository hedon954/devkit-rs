/// Load a TOML file and return a `toml::Value`.
pub fn load_toml_dynamic(input: &str) -> anyhow::Result<toml::Value> {
    let config_values = {
        let config_text = std::fs::read_to_string(input)?;
        config_text.parse::<toml::Value>()?
    };
    Ok(config_values)
}

#[cfg(test)]
mod test_load_toml_dynamic {
    use super::*;

    #[test]
    fn test_load_toml_dynamic() {
        let values = load_toml_dynamic("data/config.toml").unwrap();
        let postgresql = values
            .get("postgresql")
            .unwrap()
            .get("database")
            .unwrap()
            .as_str()
            .unwrap();
        assert_eq!(postgresql, "Rust2018");
    }
}

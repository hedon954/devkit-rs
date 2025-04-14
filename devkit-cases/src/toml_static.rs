use serde::Deserialize;

#[derive(Deserialize)]
pub struct Input {
    pub xml_file: String,
    pub json_file: String,
}
#[derive(Deserialize)]
pub struct Redis {
    pub host: String,
}
#[derive(Deserialize)]
pub struct Sqlite {
    pub db_file: String,
}
#[derive(Deserialize)]
pub struct Postgresql {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
}
#[derive(Deserialize)]
pub struct Config {
    pub input: Input,
    pub redis: Redis,
    pub sqlite: Sqlite,
    pub postgresql: Postgresql,
}

pub fn load_toml_static(input: &str) -> anyhow::Result<Config> {
    let config_text = std::fs::read_to_string(input)?;
    let config = toml::from_str(&config_text)?;
    Ok(config)
}

#[cfg(test)]
mod test_load_toml_static {
    use super::*;

    #[test]
    fn test_load_toml_static() {
        let config = load_toml_static("data/config.toml").unwrap();
        assert_eq!(config.input.xml_file, "../data/sales.xml");
        assert_eq!(config.input.json_file, "../data/sales.json");
    }
}

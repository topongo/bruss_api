use serde::{Serialize, Deserialize};
use tokio_postgres::Config;

#[derive(Serialize, Deserialize, Debug)]
pub enum BrussConfig {
    #[serde(rename = "db")]
    Db {
        host: String,
        db: String,
        user: String,
        password: String,
    }
}

impl BrussConfig {
    pub fn from_file(path: &str) -> BrussConfig {
        // print pwd
        let file = std::fs::read_to_string(path).unwrap();
        toml::from_str(&file).unwrap()
    }

    pub fn get_db_configs(&self) -> Config {
        match self {
            BrussConfig::Db { db, user, password, host } => {
                let mut config = Config::new();
                config.host(host);
                config.user(user);
                config.password(password);
                config.dbname(db);
                config
            }
        }
    }
}


use crate::error::Error;
use crate::Result;
use std::env;
use std::str::FromStr;
use std::sync::OnceLock;

pub fn config() -> &'static Config {
    static INSTANCE: OnceLock<Config> = OnceLock::new();

    INSTANCE.get_or_init(|| {
        Config::load_from_env().unwrap_or_else(|err| {
            panic!("FATAL - WHILE LOADING Config -cause: {:?}", err);
        })
    })
}

#[allow(non_snake_case)]
pub struct Config {
    // --TG
    pub ADMIN_ID: i64,
    pub TG_GROUP_ID: i64,
    // -- DB
    pub DB_URL: String,
    // -- AmoCRM
    pub AMO_CITY_URL: String,
    pub AMO_CITY_TOKEN: String,
    pub AMO_FORMAT_URL: String,
    pub AMO_FORMAT_TOKEN: String,
    // -- Profitbase
    pub PROF_CITY_URL: String,
    pub PROF_CITY_API_KEY: String,
    pub PROF_FORMAT_URL: String,
    pub PROF_FORMAT_API_KEY: String,
    // -- Schedule for worker
    pub SCHEDULE: String,
}

impl Config {
    fn load_from_env() -> Result<Config> {
        Ok(Config {
            ADMIN_ID: get_env_as_parse("TG_HANMASTER_ID")?,
            TG_GROUP_ID: get_env_as_parse("TG_GROUP_ID")?,
            DB_URL: get_env("DB_URL")?,
            AMO_CITY_URL: get_env("AMO_CITY_URL")?,
            AMO_CITY_TOKEN: get_env("AMO_CITY_TOKEN")?,
            AMO_FORMAT_URL: get_env("AMO_FORMAT_URL")?,
            AMO_FORMAT_TOKEN: get_env("AMO_FORMAT_TOKEN")?,
            PROF_CITY_URL: get_env("PROF_CITY_URL")?,
            PROF_CITY_API_KEY: get_env("PROF_CITY_API_KEY")?,
            PROF_FORMAT_URL: get_env("PROF_FORMAT_URL")?,
            PROF_FORMAT_API_KEY: get_env("PROF_FORMAT_API_KEY")?,
            SCHEDULE: get_env("SCHEDULE")?,
        })
    }
}

fn get_env(name: &'static str) -> Result<String> {
    env::var(name).map_err(|_| Error::ConfigMissingEnv(name))
}

fn get_env_as_parse<T: FromStr>(name: &'static str) -> Result<T> {
    let val = get_env(name)?;
    val.parse::<T>().map_err(|_| Error::ConfigWrongFormat(name))
}

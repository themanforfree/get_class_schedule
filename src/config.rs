use clap::{Arg, Command};
pub struct Config {
    pub username: String,
    pub password: String,
}

impl Config {
    pub fn parse() -> Config {
        let env_username = std::env::var("JWGLXT_USERNAME").unwrap_or_default();
        let env_password = std::env::var("JWGLXT_PASSWORD").unwrap_or_default();
        let matches = Command::new("jwglxt")
            .version("0.1")
            .author("themanforfree <themanforfree@gmail.com>")
            .about("get haut class schedule")
            .arg(
                Arg::new("username")
                    .short('u')
                    .long("username")
                    .value_name("USERNAME")
                    .help("username of the system")
                    .takes_value(true),
            )
            .arg(
                Arg::new("password")
                    .short('p')
                    .long("password")
                    .value_name("PASSWORD")
                    .help("password of the system")
                    .takes_value(true),
            )
            .get_matches();
        Config {
            username: matches
                .value_of("username")
                .unwrap_or(&env_username)
                .to_string(),
            password: matches
                .value_of("password")
                .unwrap_or(&env_password)
                .to_string(),
        }
    }
}

use std::env;

#[derive(Debug)]
pub struct Plug {
    pub name: String,
    pub ip: String,
    pub username: String,
    pub password: String,
}

pub fn get_plugs_from_env() -> Vec<Plug> {
    let username = env::var("SHELLY_USERNAME").expect("Missing SHELLY_USERNAME env var");
    let password = env::var("SHELLY_PASSWORD").expect("Missing SHELLY_PASSWORD env var");

    return env::var("SHELLY_PLUGS").unwrap()
        .split(",")
        .map(|x| {
            let plug= x.split("@").collect::<Vec<&str>>();
            Plug {
                name: String::from(plug[0]),
                ip: String::from(plug[1]),
                username: username.clone(),
                password: password.clone(),
            }
        })
        .collect::<Vec<Plug>>();
}


use std::env;

#[derive(Clone, Debug, Copy)]
pub struct Settings<'a> {
    pub secret_key: &'a str,
    pub num_workers: usize,
    pub port: u16,
}

pub fn get_settings() -> Settings<'static> {
    Settings {
        secret_key: Box::leak(
            env::var("SECRET_KEY")
                .expect("SECRET_KEY should be set")
                .into_boxed_str(),
        ),
        num_workers: env::var("NUM_WORKERS")
            .unwrap_or_else(|_| "4".to_string())
            .parse::<usize>()
            .expect("NUM_WORKERS should be a valid number"),
        port: env::var("PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse::<u16>()
            .expect("PORT should be a valid number"),
    }
}

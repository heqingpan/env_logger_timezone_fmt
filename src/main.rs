use env_logger::Env;

fn main() {
    println!("hello, world!");
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .init();
}
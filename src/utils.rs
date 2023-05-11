pub fn setup_logger() {
    let env = env_logger::Env::new()
        .default_filter_or("info")
        .default_write_style_or("auto");

    env_logger::init_from_env(env);
}

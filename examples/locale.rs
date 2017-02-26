use std::env;

fn main() {
    let env_lang = env::var("LANG");
    let lang = env::var("LANG").ok()
                               .and_then(|r|r.split('_')
                                             .next()
                                             .map(|l|l.to_owned())
                                        );
    println!("env(lang) {:?}", env_lang );
    println!(" -> {:?}", lang );
    
    if let Ok(env_lang) = env::var("LANG") {
        if starts_with("de") {
        }
    }
}

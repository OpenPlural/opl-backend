fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=.env.staging");
    println!("cargo:rerun-if-changed=.env.production");

    let env_type = if option_env!("STAGING") == Some("true") {
        "staging"
    } else {
        "production"
    };

    if let Ok(env) = std::fs::read_to_string(format!(".env.{env_type}")) {
        for line in env.lines() {
            if let Some((key, value)) = line.split_once("=") {
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
    }
}
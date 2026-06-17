fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rerun-if-changed=.env");

    if let Ok(env) = std::fs::read_to_string(".env") {
        for line in env.lines() {
            if let Some((key, value)) = line.split_once("=") {
                println!("cargo:rustc-env={}={}", key, value);
            }
        }
    }
}
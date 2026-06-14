fn main() {
    println!("cargo:rerun-if-changed=migrations");
    println!("cargo:rustc-env=SESSION_COOKIE_DOMAIN=webbiii.cc");
}
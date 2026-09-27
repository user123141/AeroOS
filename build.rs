fn main() {
    #[cfg(feature = "ape")]
    cosmo_build::apeify().expect("APE build failed");
    println!("cargo:rerun-if-changed=web/");
    println!("cargo:rerun-if-changed=config/aeroos.config.toml");
}

fn main() {
    #[cfg(feature = "esp")]
    embuild::espidf::sysenv::output();
}

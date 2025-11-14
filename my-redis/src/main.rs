fn main() {
    let a: Box<dyn std::error::Error + Send + Sync> = "invalid decimal number".into();
}

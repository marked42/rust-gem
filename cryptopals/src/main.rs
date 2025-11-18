use cryptopals::base64;

// https://cryptopals.com/sets/1
fn main() {
    let plain = "test";
    let cipher = base64::encode(plain);
    println!("{:?}", cipher);
}

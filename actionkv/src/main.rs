fn main() {
    let abc = b"abc";
    println!("input: {:?}", abc);
    println!("output: {:08x}", parity_bit(abc));
    println!();
    let abcd = b"abcd";
    println!("input: {:?}", abcd);
    println!("result: {:08x}", parity_bit(abcd))
}

fn parity_bit(bytes: &[u8]) -> u8 {
    let mut n_ones: u32 = 0;

    for byte in bytes {
        let ones = byte.count_ones();
        n_ones += ones;
        println!("{} (0b{:08b}) has {} one bits", byte, byte, ones);
    }

    // 返回奇偶校验位：1 表示奇数个1，0 表示偶数个1
    (n_ones % 2) as u8
}

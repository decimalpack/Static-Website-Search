pub fn naive_hash(token: &str, seed: u64) -> u64 {
    let string_hash = token
        .chars()
        .enumerate()
        .map(|(i, c)| (i + 1) * c as usize)
        .map(|x| x as u64)
        .sum::<u64>();
    (string_hash + 1) * (seed + 1)
}
fn main() {
    let h: u64 = naive_hash("", 14);
    dbg!(h);
}

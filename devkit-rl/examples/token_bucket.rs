use devkit_rl::TokenBucket;

fn main() {
    let tb = TokenBucket::new(10, 1, None);
    assert!(tb.allow())
}

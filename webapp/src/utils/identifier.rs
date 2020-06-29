use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};
use base32;

#[derive(Hash)]
pub struct IdGenerator<T> {
    timestamp: u128,
    salt: u64,
    content: T
}

impl<T> IdGenerator<T>
{
    pub fn new(t: T) -> Self {
        IdGenerator {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos()).unwrap_or(0 as u128),
            salt: rand::random::<u64>(),
            content: t
        }
    }
}

pub trait Generate {
    fn generate(self: &Self) -> String;
}

impl<T> Generate for IdGenerator<T> where T: Hash
{
    fn generate(self: &Self) -> String {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let raw = hasher.finish();
        let t = base32::encode(base32::Alphabet::Crockford, &raw.to_be_bytes());
        t.to_lowercase()
    }
}

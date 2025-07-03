use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::iter::FromIterator;
use std::net::TcpStream;
use std::sync::RwLock;

pub struct Stemmer {
    store: RwLock<HashMap<String, HashSet<String>>>,
}

impl Stemmer {
    pub fn new() -> Self {
        Stemmer {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&mut self, key: &str) -> HashSet<String> {
        // First try a read-only lookup
        if let Some(cached) = self.store.read().unwrap().get(key) {
            return cached.clone();
        }

        // Compute if not found
        let result = self.compute(key);

        // Insert into cache
        let mut store = self.store.write().unwrap();
        store.insert(key.to_string(), result.clone());

        result
    }

    fn compute(&self, word: &str) -> HashSet<String> {
        let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
        let request = format!(
            "GET /stemmer/{} HTTP/1.1\r\n\
                           Host: example.com\r\n\
                           Connection: close\r\n\
                           \r\n",
            word
        );
        stream.write_all(request.as_bytes()).unwrap();
        let mut response = Vec::new();
        stream.read_to_end(&mut response).unwrap();
        let syns = HashSet::from_iter(vec![word.to_string()]);
        syns
    }
}

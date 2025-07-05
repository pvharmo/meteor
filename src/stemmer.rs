use pyo3::ffi::c_str;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString};
use std::collections::HashMap;
use std::sync::RwLock;

pub struct Stemmer {
    store: RwLock<HashMap<String, String>>,
}

impl Stemmer {
    pub fn new() -> Self {
        Stemmer {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> String {
        self.store.read().unwrap().get(key).unwrap().clone()
    }

    pub fn get_or_compute(&mut self, key: &str) -> String {
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

    fn compute(&self, word: &str) -> String {
        let mut s = String::new();
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            locals.set_item("word", word).unwrap();
            py.run(
                c_str!(
                    r#"
from nltk.stem import PorterStemmer
stemmer = PorterStemmer()
ret = stemmer.stem(word)
                "#
                ),
                None,
                Some(&locals),
            )
            .unwrap();

            let ret = locals.get_item("ret").unwrap().unwrap();
            s = ret
                .downcast::<PyString>()
                .unwrap()
                .to_string_lossy()
                .to_string();
        });
        return s;
    }
}

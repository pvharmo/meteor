use pyo3::ffi::c_str;
use pyo3::types::{PyDict, PySet, PyString};
use std::collections::HashMap;
use std::sync::RwLock;

use pyo3::prelude::*;

pub struct Synsets {
    store: RwLock<HashMap<String, Vec<String>>>,
}

impl Synsets {
    pub fn new() -> Self {
        Synsets {
            store: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, key: &str) -> Vec<String> {
        self.store.read().unwrap().get(key).unwrap().clone()
    }

    pub fn get_or_compute(&mut self, key: &str) -> Vec<String> {
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

    fn compute(&self, word: &str) -> Vec<String> {
        let mut rust_strings = Vec::new();
        Python::with_gil(|py| {
            let locals = PyDict::new(py);
            locals.set_item("word", word).unwrap();
            py.run(
                c_str!(
                    r#"
from itertools import chain
from typing import List, Tuple

from nltk.corpus import wordnet

ret = set(
    chain.from_iterable(
        (
            lemma.name()
            for lemma in synset.lemmas()
            if lemma.name().find("_") < 0
        )
        for synset in wordnet.synsets(word)
    )
).union({word})
                "#
                ),
                None,
                Some(&locals),
            )
            .unwrap();

            let ret = locals.get_item("ret").unwrap().unwrap();
            let ret_set = ret.downcast::<PySet>().unwrap();
            rust_strings = ret_set
                .iter()
                .map(|item| {
                    let py_str: &Bound<'_, PyString> = item.downcast::<PyString>().unwrap();
                    py_str.to_string()
                })
                .collect();
        });
        return rust_strings;
    }
}

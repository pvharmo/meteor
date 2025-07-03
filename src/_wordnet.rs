use libc::{c_char, c_int, c_long, c_void};
use std::{
    ffi::{CStr, CString, c_uint},
    os::raw::c_short,
};

pub const NOUN: c_int = 1;
pub const VERB: c_int = 2;
pub const ADJ: c_int = 3;
pub const ADV: c_int = 4;
const ALL_PTRS: c_int = 0;
const SNSET: c_int = 3;

#[repr(C)]
struct Synset {
    hereiam: c_long,
    sstype: c_int,
    fnum: c_int,
    pos: *mut c_char,
    wcount: c_int,
    words: *mut *mut c_char, // Pointer to array of C strings
    lexid: *mut c_int,       // Pointer to array of integers
    wnsns: *mut c_int,       // Pointer to array of integers
    whichword: c_int,
    ptrcount: c_int,
    ptrtyp: *mut c_int,
    ptroff: *mut c_long,
    ppos: *mut c_int,
    pto: *mut c_int,
    pfrm: *mut c_int,
    fcount: c_int,
    frmid: *mut c_int,
    frmto: *mut c_int,
    defn: *mut c_char, // Pointer to definition string
    key: c_uint,
    dcount: c_int,
    nextss: *mut Synset,
    nextform: *mut Synset,
    searchtype: c_int,
    ptrlist: *mut Synset,
    headword: *mut c_char,
    headsense: *mut c_short,
}

unsafe extern "C" {
    fn wninit();
    fn findtheinfo_ds(
        searchstr: *const c_char,
        pos: c_int,
        ptr_type: c_int,
        sense: c_int,
    ) -> *mut Synset;
    fn free_syns(ptr: *mut c_void);
}

pub fn init() {
    unsafe {
        wninit();
    }
}

pub fn get_all_synsets(word: &str) -> Vec<String> {
    // let mut synsets1: Vec<String> = get_synsets_pos(word, NOUN).into_iter().flatten().collect();
    // let mut synsets2 = get_synsets_pos(word, VERB).into_iter().flatten().collect();
    // let mut synsets3 = get_synsets_pos(word, ADJ).into_iter().flatten().collect();
    // let mut synsets4 = get_synsets_pos(word, ADV).into_iter().flatten().collect();
    // synsets1.append(&mut synsets2);
    // synsets1.append(&mut synsets3);
    // synsets1.append(&mut synsets4);
    // return synsets1;
    dbg!(get_synsets_pos(word, NOUN));
    get_synsets_pos(word, NOUN).into_iter().flatten().collect()
}

pub fn get_synsets_pos(word: &str, pos: c_int) -> Vec<Vec<String>> {
    let c_word = CString::new(word).expect("CString failed");
    let mut synsets: Vec<Vec<String>> = Vec::new();

    unsafe {
        let result = findtheinfo_ds(c_word.as_ptr(), pos, ALL_PTRS, SNSET) as *mut Synset;
        let mut current = result;

        while !current.is_null() {
            let wcount = (*current).wcount as usize;
            let mut words = Vec::new();
            let words_ptr = (*current).words;

            // Only access words if we have some and pointer is valid
            if wcount > 0 && !words_ptr.is_null() {
                for i in 0..wcount {
                    let word_ptr = *words_ptr.add(i);
                    if !word_ptr.is_null() {
                        let word_cstr = CStr::from_ptr(word_ptr);
                        words.push(word_cstr.to_string_lossy().into_owned());
                    }
                }
            }

            synsets.push(words);
            current = (*current).nextss;
        }

        // Free entire linked list
        if !result.is_null() {
            free_syns(result as *mut c_void);
        }
    }

    synsets
}

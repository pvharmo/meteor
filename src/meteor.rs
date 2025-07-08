use crate::stemmer::Stemmer;
use crate::synsets::Synsets;

// Utility Types
type EnumeratedWord = (usize, String);
type MatchedTuple = (usize, usize);
type UnmatchedTuple = (usize, String);

fn generate_enums(
    hypothesis: &[&str],
    reference: &[&str],
) -> (Vec<EnumeratedWord>, Vec<EnumeratedWord>) {
    let enum_hypothesis_list: Vec<_> = hypothesis
        .iter()
        .enumerate()
        .map(|(i, &word)| (i, word.to_lowercase().to_string()))
        .collect();

    let enum_reference_list: Vec<_> = reference
        .iter()
        .enumerate()
        .map(|(i, &word)| (i, word.to_lowercase().to_string()))
        .collect();

    (enum_hypothesis_list, enum_reference_list)
}

fn match_enums(
    mut enum_hypothesis_list: Vec<EnumeratedWord>,
    mut enum_reference_list: Vec<EnumeratedWord>,
) -> (Vec<MatchedTuple>, Vec<EnumeratedWord>, Vec<EnumeratedWord>) {
    let mut word_match = Vec::with_capacity(enum_hypothesis_list.len());

    let mut i = enum_hypothesis_list.len() as isize - 1;
    while i >= 0 {
        let mut j = enum_reference_list.len() as isize - 1;
        while j >= 0 {
            if enum_hypothesis_list[i as usize].1 == enum_reference_list[j as usize].1 {
                word_match.push((
                    enum_hypothesis_list[i as usize].0,
                    enum_reference_list[j as usize].0,
                ));
                enum_hypothesis_list.remove(i as usize);
                enum_reference_list.remove(j as usize);
                break;
            }
            j -= 1;
        }
        i -= 1;
    }

    (word_match, enum_hypothesis_list, enum_reference_list)
}

fn enum_stem_match(
    mut enum_hypothesis_list: Vec<EnumeratedWord>,
    mut enum_reference_list: Vec<EnumeratedWord>,
    stemmer: &Stemmer,
) -> (Vec<MatchedTuple>, Vec<UnmatchedTuple>, Vec<UnmatchedTuple>) {
    // let stemmed_enum_hypothesis = enum_hypothesis_list
    //     .into_iter()
    //     .map(|(i, word)| (i, stemmer.get(&word)))
    //     .collect::<Vec<_>>();

    // let stemmed_enum_reference = enum_reference_list
    //     .into_iter()
    //     .map(|(i, word)| (i, stemmer.get(&word)))
    //     .collect::<Vec<_>>();

    for (_, word) in enum_hypothesis_list.iter_mut() {
        *word = stemmer.get(&word);
    }

    for (_, word) in enum_reference_list.iter_mut() {
        *word = stemmer.get(&word);
    }

    match_enums(enum_hypothesis_list, enum_reference_list)
}

// fn f_hypothesis_syns(word: &str, cache: &mut Cache) -> HashSet<String> {
//     if let Some(syns) = cache.get(word) {
//         return syns.clone();
//     }

//     let mut stream = TcpStream::connect("127.0.0.1:8000").unwrap();
//     let request = format!(
//         "GET /{} HTTP/1.1\r\n\
//                        Host: example.com\r\n\
//                        Connection: close\r\n\
//                        \r\n",
//         word
//     );
//     stream.write_all(request.as_bytes()).unwrap();
//     let mut response = Vec::new();
//     stream.read_to_end(&mut response).unwrap();
//     let syns = HashSet::from_iter(vec![word.to_string()]);
//     cache.insert(word.to_string(), syns.clone());
//     syns
// }

fn enum_wordnetsyn_match(
    mut enum_hypothesis_list: Vec<EnumeratedWord>,
    mut enum_reference_list: Vec<EnumeratedWord>,
    synsets: &Synsets,
) -> (Vec<MatchedTuple>, Vec<EnumeratedWord>, Vec<EnumeratedWord>) {
    let mut word_match = vec![];

    let mut i = enum_hypothesis_list.len() as isize - 1;
    while i >= 0 {
        // Placeholder for synonym generation
        let hypothesis_syns: Vec<String> = synsets.get(&enum_hypothesis_list[i as usize].1);

        let mut j = enum_reference_list.len() as isize - 1;
        while j >= 0 {
            if hypothesis_syns.contains(&enum_reference_list[j as usize].1) {
                word_match.push((
                    enum_hypothesis_list[i as usize].0,
                    enum_reference_list[j as usize].0,
                ));
                enum_hypothesis_list.remove(i as usize);
                enum_reference_list.remove(j as usize);
                break;
            }
            j -= 1;
        }
        i -= 1;
    }

    (word_match, enum_hypothesis_list, enum_reference_list)
}

fn enum_align_words(
    enum_hypothesis_list: Vec<EnumeratedWord>,
    enum_reference_list: Vec<EnumeratedWord>,
    synsets: &Synsets,
    stemmer: &Stemmer,
) -> (Vec<MatchedTuple>, Vec<UnmatchedTuple>, Vec<UnmatchedTuple>) {
    let (exact_matches, enum_hypothesis, enum_reference) =
        match_enums(enum_hypothesis_list, enum_reference_list);

    let (stem_matches, more_hypothesis, more_ref) =
        enum_stem_match(enum_hypothesis, enum_reference, stemmer);

    let (wns_matches, remaining_hypothesis, remaining_ref) =
        enum_wordnetsyn_match(more_hypothesis, more_ref, synsets);

    let mut all_matches = vec![exact_matches, stem_matches, wns_matches].concat();
    all_matches.sort_by_key(|(i, _)| *i);

    (all_matches, remaining_hypothesis, remaining_ref)
}

fn count_chunks(matches: &[(usize, usize)]) -> usize {
    if matches.is_empty() {
        return 0;
    }

    let mut chunks = 1;
    for i in 0..matches.len() - 1 {
        let (i0, i1) = matches[i];
        let (j0, j1) = matches[i + 1];
        if j0 == i0 + 1 && j1 == i1 + 1 {
            continue;
        }
        chunks += 1;
    }

    chunks
}

pub fn meteor_score(
    reference: &Vec<&str>,
    hypothesis: &Vec<&str>,
    alpha: f64,
    beta: f64,
    gamma: f64,
    synsets: &Synsets,
    stemmer: &Stemmer,
) -> f64 {
    let (enum_hypothesis, enum_reference) = generate_enums(hypothesis, reference);
    let translation_length = enum_hypothesis.len();
    let reference_length = enum_reference.len();

    let (matches, _, _) = enum_align_words(enum_hypothesis, enum_reference, synsets, stemmer);
    let matches_count = matches.len();

    if matches_count == 0 {
        return 0.0;
    }

    let precision = matches_count as f64 / translation_length as f64;
    let recall = matches_count as f64 / reference_length as f64;

    let fmean = (precision * recall) / (alpha * precision + (1.0 - alpha) * recall);

    let chunk_count = count_chunks(&matches) as f64;
    let frag_frac = chunk_count / matches_count as f64;
    let penalty = gamma * frag_frac.powf(beta);

    (1.0 - penalty) * fmean
}

/* ########################################################################################## */

pub fn init_cache(
    hypothesis: &Vec<&str>,
    synsets: &mut Synsets,
    stemmer: &mut Stemmer,
) -> Result<(), Box<dyn std::error::Error>> {
    let reference = vec![""];
    let (enum_hypothesis, _) = generate_enums(hypothesis, &reference);

    init_enum_align_words(enum_hypothesis, synsets, stemmer);
    Ok(())
}

fn init_enum_align_words(
    enum_hypothesis_list: Vec<EnumeratedWord>,
    synsets: &mut Synsets,
    stemmer: &mut Stemmer,
) {
    let h: Vec<EnumeratedWord> = enum_hypothesis_list
        .into_iter()
        .map(|(i, word)| (i, stemmer.get_or_compute(&word)))
        .collect();

    h.iter().for_each(|(_, val)| {
        synsets.get_or_compute(val);
    });
}

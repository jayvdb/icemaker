use std::collections::HashMap;
use std::path::PathBuf;

use rand::Rng;
use tree_sitter::{Parser, Tree};
use tree_splicer::splice::{splice, Config};

// read a file from a path and splice-fuzz it returning a set of String that we built from it
// pub(crate) fn splice_file(hm: &HashMap<String, (Vec<u8>, Tree)>) -> Vec<String> {
pub(crate) fn splice_file(path: &PathBuf) -> Vec<String> {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

    const INTER_SPLICES_RANGE: std::ops::Range<usize> = 2..30;
    let random_inter_splices = rng.gen_range(INTER_SPLICES_RANGE);
    let random_seed = rng.gen::<u64>();
    let pd = path.display();

    eprintln!(
        "slice_file: random_inter_splices: '{random_inter_splices}', random_seed: '{random_seed}', path: {pd} "
    );

    let splicer_cfg: Config = Config {
        inter_splices: random_inter_splices,
        seed: random_seed,
        tests: 1000, // 10
        //
        chaos: 10,
        deletions: 5,
        node_types: tree_splicer::node_types::NodeTypes::new(tree_sitter_rust::NODE_TYPES).unwrap(),
        language: tree_sitter_rust::language(),
        max_size: 200_000,
        // do not reparse for now?
        reparse: 200,
    };

    let file_content = std::fs::read_to_string(path)
        .unwrap_or_else(|_| panic!("splicer failed to read file {}", path.display()));
    // skip if its too long to avoid stack overflows somewhere
    // TODO use ignore_files_for_splicing() here
    if file_content.lines().count() > 1000 {
        return Vec::new();
    }

    let mut parser = Parser::new();
    // rust!
    parser.set_language(tree_sitter_rust::language()).unwrap();

    let tree = parser.parse(&file_content, None);

    let mut hm = HashMap::new();
    hm.insert(
        path.display().to_string(),
        (file_content.into_bytes(), tree.unwrap()),
    );

    // TODO just return Iterator here
    splice(splicer_cfg, &hm)
        .map(|f| String::from_utf8(f).unwrap_or_default())
        .collect::<Vec<String>>()
}

pub(crate) fn splice_file_from_set(
    //  path: &PathBuf,
    hmap: &HashMap<String, (Vec<u8>, Tree)>,
) -> Vec<String> {
    let mut rng: rand::prelude::ThreadRng = rand::thread_rng();

    const INTER_SPLICES_RANGE: std::ops::Range<usize> = 2..30;
    let random_inter_splices = rng.gen_range(INTER_SPLICES_RANGE);
    let random_seed = rng.gen::<u8>() as u64;

    eprintln!(
        "slice_file: random_inter_splices: '{random_inter_splices}', random_seed: '{random_seed}'"
    );

    let splicer_cfg: Config = Config {
        inter_splices: random_inter_splices,
        seed: random_seed,
        tests: 1000, // 10
        //
        chaos: 10,    // % chance that a chaos mutation will occur
        deletions: 0, //
        node_types: tree_splicer::node_types::NodeTypes::new(tree_sitter_rust::NODE_TYPES).unwrap(),
        language: tree_sitter_rust::language(),
        max_size: 1048576,
        // do not reparse for now?
        reparse: 1048576,
    };

    // it seems that with this approach, we no longer have the notion of "files", we just have one big set of input and are able to generate random ouputs from it

    /*
        let mut parser = Parser::new();
        // rust!
        parser.set_language(tree_sitter_rust::language()).unwrap();

        let file_content = std::fs::read_to_string(path)
            .expect(&format!("splicer failed to read file {}", path.display()));
    */
    //  let tree = parser.parse(&file_content, None);

    // TODO just return Iterator here

    // TODO tree splicer sometimes just hangs.
    splice(splicer_cfg, hmap)
        .map(|f| String::from_utf8(f).unwrap_or_default())
        .collect::<Vec<String>>()
}

// such files will most likely just causes known crashes or hang the splicing
pub(crate) fn ignore_file_for_splicing(file: &PathBuf) -> bool {
    const LINE_LIMIT: usize = 400; // was 1000

    let content = std::fs::read_to_string(file).unwrap_or_default();
    let lines_count = content.lines().count();

    lines_count > LINE_LIMIT
        || content.contains("no_core")
        || content.contains("lang_items")
        || content.contains("mir!(")
        || content.contains("break rust")
        || (content.contains("failure-status: 101") && content.contains("known-bug"))
        // if the file is in an "icemaker" dir, do not use it for fuzzing...
        || file.display().to_string().contains("icemaker")
}

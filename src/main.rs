extern crate core;

use std::borrow::{Borrow, BorrowMut};
// use serde_json::Value;
use serde::{Serialize, Deserialize};
use indextree::Arena;
use std::env;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
enum JoinType {
    Inner,
    LeftOuter,
    RightOuter,
    FullOuter,
}

#[derive(Serialize, Deserialize, Debug)]
struct Attribute {
    attr_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Equalizer {
    left_attr: Attribute,
    right_attr: Attribute,
}


// Force to rename "extra-info" into "extra_info"
#[derive(Serialize, Deserialize, Debug)]
struct JoinAttr {
    join_type : JoinType,
    equalizers : Vec<Equalizer>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ScanAttr {
    table_name : String,
    attributes : Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
enum NodeAttr {
    Join(JoinAttr),
    Scan(ScanAttr),
}

#[derive(Serialize, Deserialize, Debug)]
struct TreeOp {
    name: String,
    cardinality: i32,
    extra_info: String,
    children: Vec<Box<TreeOp>>,
    attr: Option<NodeAttr>,
}

fn traverse(node: &mut TreeOp) {

    match node.name.as_str() {
        "HASH_JOIN" => {
            let tmp_extra_info : String = node.extra_info.replace("[INFOSEPARATOR]", "");
            let tmp_strs: Vec<&str> = tmp_extra_info.split("\n").collect();
            let info_strs: Vec<&str> = tmp_strs.into_iter().filter(|&s| s.len() != 0).collect();
            let join_type_str: &str = info_strs.first().expect("Failed to Get Type");
            let join_type : JoinType = match join_type_str {
                "INNER" => JoinType::Inner,
                _ => panic!("Fail to parse Join Type {}", tmp_extra_info)
            };
            node.attr = Some(NodeAttr::Join(JoinAttr { join_type: join_type, equalizers: vec![] }));
        },
        "SEQ_SCAN" => {
            let tmp_extra_info : String = node.extra_info.replace("[INFOSEPARATOR]", "");
            let tmp_strs: Vec<&str> = tmp_extra_info.split("\\n").collect();
            let info_strs: Vec<&str> = tmp_strs.into_iter().filter(|&s| s.len() != 0).collect();
            let table_name : String = info_strs.first().expect("Failed to Get Table").to_string();
            node.attr = Some(NodeAttr::Scan(ScanAttr { table_name: table_name, attributes: vec![] }));
        },
        _ => (),
    }

    for child_node in node.children.iter_mut() {
        traverse(child_node);
    }
}

fn main() {

    let args: Vec<String>  = env::args().collect();

    let filename = &args[1];

    let contents = fs::read_to_string(filename).expect(format!("Cannot read file {}", filename).as_str());

    // Parse the string of data into serde_json::Value.
    let mut root: TreeOp = serde_json::from_str(contents.as_str()).expect("Failed to Parse Json!");

    // Access parts of the data by indexing with square brackets.
    // println!("Query is {}", root.extra_info.to_string().replace("\\n", "\n"));

    let result_collector: &[Box<TreeOp>]  = root.children.borrow();
    println!("Collector is {}", result_collector.len());

    traverse(root.borrow_mut());
    // Create a new arena
    let arena = &mut Arena::new();

    // Add some new nodes to the arena
    let a = arena.new_node(1);
    let b = arena.new_node(2);

    // Append a to b
    assert!(a.checked_append(b, arena).is_ok());
    assert_eq!(b.ancestors(arena).into_iter().count(), 2);
}

// Waive
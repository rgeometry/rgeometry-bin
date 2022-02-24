// use quick_xml::events::attributes::Attributes;
// use quick_xml::events::Event;
// use quick_xml::Reader;

use osmpbf::elements::Element;
use osmpbf::mmap_blob::Mmap;
use osmpbf::reader::ElementReader;
use osmpbf::BlobDecode;
// use osmpbf::BlobReader;
// use osmpbf::{Element, ElementReader};
// use std::collections::HashMap;
// use std::collections::HashSet;
// use std::fs::File;
// use std::io::BufReader;
// use std::io::Read;
// use std::io::Seek;
// use std::io::Write;

// use std::str::FromStr;

// use rusqlite::{params, Connection};

// use std::sync::Mutex;

// use anes::*;
use anyhow::Result;
use log::info;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::io::Write;
use std::path::Path;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
// use std::path::Path;

use crate::node_db;
use crate::status::*;

// mod indexed;
// use indexed::*;

// mod gui;

fn analyze_osm_pbf(path: impl AsRef<Path>) -> Result<(u64, u64)> {
    let mmap = unsafe { Mmap::from_path(path)? };

    let ways = AtomicU64::new(0);
    let lines = AtomicU64::new(0);
    let n_count = AtomicU64::new(0);
    // let n_blocks = AtomicU64::new(0);

    info!("Blocks: {}", mmap.blob_iter().count());

    let status = Status::new(mmap.blob_iter().count() as u64);

    mmap.blob_iter().par_bridge().for_each(|blob| {
        // print!(
        //     "\r{}Progress: {} {} {}",
        //     ClearLine::All,
        //     n_blocks.load(Ordering::Relaxed),
        //     n_count.load(Ordering::Relaxed),
        //     ways.load(Ordering::Relaxed),
        // );
        std::io::stdout().flush().unwrap();
        let blob = blob.unwrap();
        if let BlobDecode::OsmData(block) = blob.decode().unwrap() {
            for group in block.groups() {
                for way in group.ways() {
                    let nodes: Vec<i64> = way.refs().collect();
                    for &node in &nodes {
                        n_count.fetch_max(node as u64, Ordering::Relaxed);
                    }
                    if nodes.len() >= 3 && nodes[0] == *nodes.last().unwrap() {
                        // ways += 1;
                        ways.fetch_add(1, Ordering::Relaxed);
                    } else {
                        // lines += 1;
                        lines.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
        // n_blocks.fetch_add(1, Ordering::Relaxed);
        status.add(1);
    });

    // info!("Polygons: {}", ways.load(Ordering::Relaxed));
    // info!("Lines:    {}", lines.load(Ordering::Relaxed));
    // info!("Nodes:    {}", n_count.load(Ordering::Relaxed));
    Ok((
        n_count.load(Ordering::Relaxed),
        ways.load(Ordering::Relaxed),
    ))
}

fn _create_polygons() -> Result<()> {
    info!("Creating polygons");
    let mmap = unsafe { Mmap::from_path("planet.osm.pbf")? };

    let status = Status::new(mmap.blob_iter().count() as u64);

    let ways = AtomicU64::new(0);

    mmap.blob_iter().par_bridge().for_each(|blob| {
        let blob = blob.unwrap();
        if let BlobDecode::OsmData(block) = blob.decode().unwrap() {
            for group in block.groups() {
                for way in group.ways() {
                    let nodes: Vec<i64> = way.refs().collect();
                    // for &node in &nodes {
                    //     n_count.fetch_max(node as u64, Ordering::Relaxed);
                    // }
                    if nodes.len() >= 3 && nodes[0] == *nodes.last().unwrap() {
                        // ways += 1;
                        ways.fetch_add(1, Ordering::Relaxed);
                    } else {
                        // lines += 1;
                        // lines.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        }
        status.add(1);
    });

    // info!("Polygons: {}", ways.load(Ordering::Relaxed));
    // info!("Lines:    {}", lines.load(Ordering::Relaxed));
    // info!("Nodes:    {}", n_count.load(Ordering::Relaxed));

    // info!("Indexing");
    // let i = Indexed::new(mmap);
    // let block_counter = AtomicU64::new(0);
    // let ways = AtomicU64::new(0);
    // let lines = AtomicU64::new(0);
    // let node_count = AtomicU64::new(0);
    // info!("Parsing ways");
    // // i.way_blocks().par_bridge().for_each(|offset| {
    // i.way_blocks().for_each(|offset| {
    //     info!("Parsing block: {:?}", offset);
    //     let block = i.read_block(offset);
    //     let n = block_counter.fetch_add(1, Ordering::Relaxed) + 1;
    //     // print!("\r{}Parsing block: {}", ClearLine::All, n);
    //     std::io::stdout().flush().unwrap();
    //     let mut set = HashSet::new();
    //     for elt in block.elements() {
    //         if let Element::Way(way) = elt {
    //             let nodes: Vec<i64> = way.refs().collect();
    //             if nodes.len() >= 3 && nodes[0] == *nodes.last().unwrap() {
    //                 // ways += 1;
    //                 ways.fetch_add(1, Ordering::Relaxed);
    //                 for node_id in nodes {
    //                     let offset = i.lookup_node_offset(node_id);
    //                     set.insert(offset.0);
    //                     // i.lookup_node(node_id);
    //                     // node_count.fetch_add(1, Ordering::Relaxed);
    //                 }
    //             } else {
    //                 // lines += 1;
    //                 lines.fetch_add(1, Ordering::Relaxed);
    //             }
    //         }
    //     }
    //     info!("Touched blocks:    {}", set.len());
    //     // ways += block.elements().count();
    // });
    // println!("");
    // info!("Polygons: {}", ways.load(Ordering::Relaxed));
    // info!("Nodes:    {}", node_count.load(Ordering::Relaxed));
    // info!("Lines:    {}", lines.load(Ordering::Relaxed));
    Ok(())
}

pub fn test_db() -> Result<()> {
    info!("TestDB");

    // let reader = ElementReader::from_path("planet-210201.osm.pbf")?;

    let db = node_db::new(Path::new("nodes.db"), 9424200057);

    let (lat, lon) = db.lookup(100);
    dbg!(db.lookup(100));
    db.set(100, 0, 0);
    dbg!(db.lookup(100));
    db.set(100, lat, lon);
    dbg!(db.lookup(100));
    // dbg!(db.lookup(10));
    // dbg!(db.lookup(20));
    // db.set(20, 14276935300, -11051916300);

    Ok(())
}

pub fn create_db(path: impl AsRef<Path>) -> Result<()> {
    info!("Analyzing data file");
    let (n_nodes, _n_polygons) = analyze_osm_pbf(&path)?;

    info!("CreateDB");

    let reader = ElementReader::from_path(path)?;

    let db = node_db::new(Path::new("nodes.db"), n_nodes);

    {
        let status = Status::new(n_nodes);
        reader.par_map_reduce(
            |element| {
                if let Element::DenseNode(node) = element {
                    status.add(1);
                    db.set(node.id, node.nano_lat(), node.nano_lon())
                }
            },
            || (),
            |_, _| (),
        )?;
    }

    Ok(())
}

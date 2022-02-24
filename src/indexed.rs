use osmpbf::mmap_blob::*;
// use osmpbf::BlobReader;
use osmpbf::Element::*;
use osmpbf::{BlobDecode, ByteOffset, PrimitiveBlock};
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::ops::RangeInclusive;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::sync::Mutex;

use anes::*;
use log::{debug, info};

struct Block {
    nodes: RangeInclusive<i64>,
    ways: RangeInclusive<i64>,
    // relations: IdRange,
}

pub struct Indexed {
    mmap: Mmap,
    blocks: Vec<(ByteOffset, Block)>,
    cache: VecBuffer<(ByteOffset, Arc<PrimitiveBlock>)>,
    nodes: VecBuffer<(ByteOffset, Arc<HashMap<i64, (i64, i64)>>)>,
}

impl Indexed {
    pub fn new(mmap: Mmap) -> Indexed {
        let counter = AtomicU64::new(0);
        let blocks = Mutex::new(Vec::new());
        let block_count = mmap.blob_iter().count();
        mmap.blob_iter().par_bridge().for_each(|blob| {
            let prev = counter.fetch_add(1, Ordering::Relaxed);
            print!(
                "\r{}Progress: {} / {}",
                ClearLine::All,
                prev + 1,
                block_count
            );
            std::io::stdout().flush().unwrap();
            let blob = blob.unwrap();
            match blob.decode().unwrap() {
                BlobDecode::OsmData(block) => {
                    // let prev = blocks.fetch_add(1, Ordering::Relaxed);
                    // println!("Blocks: {}", prev + 1);
                    let mut node_range = RangeInclusive::new(1, 0);
                    let mut way_range = RangeInclusive::new(1, 0);
                    for elt in block.elements() {
                        if let DenseNode(dense) = elt {
                            upd_range(&mut node_range, dense.id);
                        } else if let Way(way) = elt {
                            upd_range(&mut way_range, way.id())
                        }
                    }
                    if !node_range.is_empty() {
                        debug!(
                            "Block: {:?}, nodes: {:?}, ways: {:?}",
                            blob.offset(),
                            &node_range,
                            &way_range
                        );
                    }
                    blocks.lock().unwrap().push((
                        blob.offset(),
                        Block {
                            nodes: node_range,
                            ways: way_range,
                        },
                    ));
                    // ()
                }
                // Err(e) => ()
                _ => (),
            }
        });
        println!("");
        std::io::stdout().flush().unwrap();
        Indexed {
            mmap: mmap,
            blocks: blocks.into_inner().unwrap(),
            cache: VecBuffer::with_capacity(10),
            nodes: VecBuffer::with_capacity(100),
        }
    }

    pub fn lookup_node(&self, node_id: i64) -> (i64, i64) {
        for (offset, block) in self.blocks.iter() {
            if block.nodes.contains(&node_id) {
                let map = self.read_node_map(*offset);
                return *map.get(&node_id).expect("Node id not found in block");
            }
        }
        panic!("Node id not found in file")
    }

    pub fn lookup_node_offset(&self, node_id: i64) -> ByteOffset {
        for (offset, block) in self.blocks.iter() {
            if block.nodes.contains(&node_id) {
                return *offset;
            }
        }
        panic!("Node id not found in file")
    }

    fn read_node_map(&self, offset: ByteOffset) -> Arc<HashMap<i64, (i64, i64)>> {
        let cached = self.nodes.lookup(|elt| elt.0 == offset);
        match cached {
            Some(ret) => ret.1,
            None => {
                let block = self.read_block(offset);
                let mut map = HashMap::new();
                for elt in block.elements() {
                    match elt {
                        Node(node) => {
                            map.insert(node.id(), (node.nano_lat(), node.nano_lon()));
                        }
                        DenseNode(node) => {
                            map.insert(node.id, (node.nano_lat(), node.nano_lon()));
                        }
                        Way(_) => {}
                        Relation(_) => {}
                    }
                }
                let rc = Arc::new(map);
                self.nodes.push((offset, rc.clone()));
                rc
            }
        }
    }

    pub fn read_block(&self, offset: ByteOffset) -> Arc<PrimitiveBlock> {
        let cached = self.cache.lookup(|elt| elt.0 == offset);
        match cached {
            Some(ret) => {
                info!("Got block from cache: {:?}", offset);
                ret.1
            }
            None => {
                info!("Loading block: {:?}", offset);
                let mut iter = self.mmap.blob_iter();
                iter.seek(offset);
                match iter.next().unwrap().unwrap().decode().unwrap() {
                    BlobDecode::OsmData(block) => {
                        let rc = Arc::new(block);
                        // self.cache.push((offset, rc.clone()));
                        rc
                    }
                    _ => panic!(),
                }
            }
        }
    }

    pub fn way_blocks(&self) -> impl Iterator<Item = ByteOffset> + Send + Sync + '_ {
        self.blocks
            .iter()
            .filter(|(_offset, block)| !block.ways.is_empty())
            .map(move |(offset, _block)| *offset)
    }
}

fn upd_range<T: Copy + Ord>(range: &mut RangeInclusive<T>, value: T) {
    if range.is_empty() {
        *range = RangeInclusive::new(value, value)
    } else {
        *range = RangeInclusive::new(
            std::cmp::min(*range.start(), value),
            std::cmp::max(*range.end(), value),
        )
    }
}

struct VecBuffer<T> {
    max_size: usize,
    vec: Mutex<Vec<T>>,
}

impl<T: Clone> VecBuffer<T> {
    pub fn with_capacity(capacity: usize) -> VecBuffer<T> {
        VecBuffer {
            max_size: capacity,
            vec: Mutex::new(Vec::with_capacity(capacity)),
        }
    }

    pub fn lookup(&self, cond: impl Fn(&T) -> bool) -> Option<T> {
        let mut vec = self.vec.lock().unwrap();
        if vec.is_empty() {
            return None;
        }
        for n in 0..vec.len() - 1 {
            if cond(&vec[n]) {
                vec.swap(0, n);
                return Some(vec[0].clone());
            }
        }
        return None;
    }

    pub fn push(&self, value: T) {
        let mut vec = self.vec.lock().unwrap();
        while vec.len() >= self.max_size {
            vec.pop();
        }
        vec.insert(0, value)
    }

    // pub fn buffered(&mut self, cond: impl Fn(&T) -> bool, gen: impl Fn() -> T) -> &T {
    //     for n in 0..self.vec.len() {
    //         if cond(&self.vec[0]) {
    //             let block = self.vec.remove(n);
    //             self.vec.insert(0, block);
    //             return &self.vec[0];
    //         }
    //     }
    //     let elt = gen();
    //     while self.vec.len() >= self.max_size {
    //         self.vec.pop();
    //     }
    //     self.vec.insert(0, elt);
    //     return &self.vec[0];
    // }
}

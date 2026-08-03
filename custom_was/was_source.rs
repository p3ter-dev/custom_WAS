use mork_expr::{destruct, item_byte, Expr, Tag};
use pathmap::zipper::PrefixZipper;
use std::iter;

use crate::sources::{AFactor, Resource, ResourceRequest, Source};
use crate::custom_was::sampling_zipper::WeightedSamplingZipper;

pub struct WASSource {
    pub e: Expr,
    pub temperature: f32,
}

impl Source for WASSource {
    fn new(e: Expr) -> Self {
        let mut temp = 1.0f32;
        destruct!(e, ("WAS" _pattern {t: f32}), {
            temp = t;
        }, _err => {});

        WASSource { e, temperature: temp }
    }

    fn request(&self) -> impl Iterator<Item = ResourceRequest> {
        iter::once(ResourceRequest::BTM([].as_slice()))
    }

    fn source<'trie, 'path, It: Iterator<Item = Resource<'trie, 'path>>>(
        &self,
        mut it: It,
    ) -> AFactor<'trie, ()>
    where
        'path: 'trie,
    {
        static WAS_PREFIX: [u8; 5] = [
            item_byte(Tag::Arity(2)),
            item_byte(Tag::SymbolSize(3)),
            b'W', b'A', b'S',
        ];

        let Resource::BTM(rz) = it.next().unwrap() else {
            unreachable!()
        };

        // Wrap the untracked zipper into our sampling wrapper
        let sampled_rz = WeightedSamplingZipper::new(rz, self.temperature);
        let prefix_rz = PrefixZipper::new(&WAS_PREFIX[..], sampled_rz);

        AFactor::WeightedPosSource(prefix_rz)
    }
}


impl WASSource {
    pub fn new(expr: Expr) -> Self {
        Self {
            e: expr,
            temperature: 1.0, // Or default temperature parameter if needed
        }
    }

    pub fn with_temperature(expr: Expr, temperature: f32) -> Self {
        Self {
            e: expr,
            temperature,
        }
    }
}

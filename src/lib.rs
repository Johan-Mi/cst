#![forbid(unsafe_code)]
#![deny(
    unused_results,
    clippy::let_underscore_untyped,
    clippy::allow_attributes,
    clippy::allow_attributes_without_reason
)]
#![warn(clippy::nursery, clippy::pedantic)]
#![expect(
    clippy::must_use_candidate,
    reason = "redundant because of `unused_results`"
)]

use codemap::Span;
use std::ops::Range;

pub struct Tree<Kind> {
    kind: Vec<Kind>,
    span: Vec<Span>,
    child_indices: Vec<Range<Index>>,
    children: Vec<Node<Kind>>,
}

pub struct Node<Kind> {
    index: Index,
    typed: std::marker::PhantomData<Kind>,
}

impl<Kind> Clone for Node<Kind> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Kind> Copy for Node<Kind> {}

impl<Kind> Node<Kind> {
    pub fn kind(self, tree: &Tree<Kind>) -> Kind
    where
        Kind: Copy,
    {
        tree.kind[usize(self.index)]
    }

    pub fn span(self, tree: &Tree<Kind>) -> Span {
        tree.span[usize(self.index)]
    }

    pub fn children(self, tree: &Tree<Kind>) -> &[Self] {
        let range = &tree.child_indices[usize(self.index)];
        &tree.children[usize(range.start)..usize(range.end)]
    }
}

pub struct Builder<Kind>(std::marker::PhantomData<Kind>);

impl<Kind> Builder<Kind> {
    pub fn start_node(&mut self, kind: Kind) {
        todo!()
    }

    pub fn finish_node(&mut self) {
        todo!()
    }

    pub fn checkpoint(&self) -> Checkpoint {
        todo!()
    }

    pub fn finish_node_at(&mut self, checkpoint: Checkpoint, kind: Kind) {
        todo!()
    }

    pub fn token(&mut self, kind: Kind, span: Span) {
        todo!()
    }

    pub fn build(self) -> (Tree<Kind>, Node<Kind>) {
        todo!()
    }
}

pub struct Checkpoint;

type Index = u32;

const fn usize(index: Index) -> usize {
    const {
        assert!(size_of::<Index>() <= size_of::<usize>());
    }
    index as usize
}

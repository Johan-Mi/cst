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

pub struct Tree<Kind>(std::marker::PhantomData<Kind>);

pub struct Node<Kind>(std::marker::PhantomData<Kind>);

impl<Kind> Clone for Node<Kind> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Kind> Copy for Node<Kind> {}

impl<Kind> Node<Kind> {
    pub fn kind(self, tree: &Tree<Kind>) -> Kind {
        todo!()
    }

    pub fn span(self, tree: &Tree<Kind>) -> Span {
        todo!()
    }

    pub fn children(self, tree: &Tree<Kind>) -> &[Self] {
        todo!()
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

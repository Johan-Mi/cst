#![no_std]
#![expect(
    clippy::must_use_candidate,
    reason = "redundant because of `unused_results`"
)]

extern crate alloc;

use alloc::vec::Vec;
use codemap::Span;

pub struct Tree<Kind> {
    kinds: Vec<Kind>,
    spans: Vec<Span>,
    sizes: Vec<Index>,
}

impl<Kind> Tree<Kind> {
    pub const fn root(&self) -> Node<'_, Kind> {
        Node {
            index: 0,
            tree: self,
        }
    }

    fn children(&self, parent: Index) -> impl Iterator<Item = Index> {
        let mut next = parent + 1;
        let end = parent + self.sizes[usize(parent)];
        core::iter::from_fn(move || {
            (next != end).then(|| (next, next += self.sizes[usize(next)]).0)
        })
    }
}

pub struct Node<'tree, Kind> {
    index: Index,
    tree: &'tree Tree<Kind>,
}

impl<Kind> Clone for Node<'_, Kind> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Kind> Copy for Node<'_, Kind> {}

impl<Kind> Node<'_, Kind> {
    pub fn kind(self) -> Kind
    where
        Kind: Copy,
    {
        self.tree.kinds[usize(self.index)]
    }

    pub fn span(self) -> Span {
        self.tree.spans[usize(self.index)]
    }

    pub fn children(self) -> impl Iterator<Item = Self> {
        let tree = self.tree;
        tree.children(self.index).map(|index| Self { index, tree })
    }

    pub fn pre_order(self) -> impl Iterator<Item = Self> {
        let tree = self.tree;
        let count = tree.sizes[usize(self.index)];
        (self.index..self.index + count).map(|index| Self { index, tree })
    }
}

pub struct Builder<Kind> {
    tree: Tree<Kind>,
    stack: Vec<usize>,
}

impl<Kind> Default for Builder<Kind> {
    fn default() -> Self {
        Self {
            tree: Tree {
                kinds: Vec::new(),
                spans: Vec::new(),
                sizes: Vec::new(),
            },
            stack: Vec::new(),
        }
    }
}

impl<Kind> Builder<Kind> {
    /// The span is expanded as child nodes are added.
    ///
    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn start_node(&mut self, kind: Kind, initial_span: Span) {
        assert!(self.tree.kinds.len() < usize(Index::MAX) - 1);
        self.stack.push(self.tree.kinds.len());
        self.tree.kinds.push(kind);
        self.tree.spans.push(initial_span);
        self.tree.sizes.push(0); // Dummy value which gets replaced when the node is closed.
    }

    /// # Panics
    ///
    /// Panics if no node has been started.
    pub fn finish_node(&mut self) {
        let entry = self.stack.pop().unwrap();
        self.tree.sizes[entry] = (self.tree.kinds.len() - entry).try_into().unwrap();
    }

    pub fn checkpoint(&self, initial_span: Span) -> Checkpoint {
        #[expect(
            clippy::missing_panics_doc,
            reason = "length is checked when pushing entries"
        )]
        Checkpoint {
            index: self.tree.kinds.len().try_into().unwrap(),
            initial_span,
        }
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn finish_node_at(&mut self, checkpoint: Checkpoint, kind: Kind) {
        assert!(self.tree.kinds.len() < usize(Index::MAX) - 1);
        let index = usize(checkpoint.index);
        self.tree.kinds.insert(index, kind);
        self.tree.spans.insert(index, checkpoint.initial_span);
        let size = Index::try_from(self.tree.kinds.len()).unwrap() - checkpoint.index + 1;
        self.tree.sizes.insert(index, size);
        let mut span = checkpoint.initial_span;
        for child in self.tree.children(checkpoint.index) {
            span = span.merge(self.tree.spans[usize(child)]);
        }
        self.tree.spans[usize(checkpoint.index)] = span;
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn token(&mut self, kind: Kind, span: Span) {
        assert!(self.tree.kinds.len() < usize(Index::MAX) - 1);
        for &parent in &self.stack {
            let parent_span = &mut self.tree.spans[parent];
            *parent_span = parent_span.merge(span);
        }
        self.tree.kinds.push(kind);
        self.tree.spans.push(span);
        self.tree.sizes.push(1);
    }

    /// # Panics
    ///
    /// Panics if there are unfinished nodes.
    pub fn build(self) -> Tree<Kind> {
        assert!(self.stack.is_empty());
        self.tree
    }
}

#[derive(Clone, Copy)]
pub struct Checkpoint {
    index: Index,
    initial_span: Span,
}

type Index = u32;

const fn usize(index: Index) -> usize {
    const { assert!(size_of::<Index>() <= size_of::<usize>()) }
    index as usize
}

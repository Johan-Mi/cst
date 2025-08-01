#![no_std]
#![expect(
    clippy::must_use_candidate,
    reason = "redundant because of `unused_results`"
)]

extern crate alloc;

use alloc::vec::Vec;
use codemap::Span;

pub struct Tree<Kind> {
    entries: Vec<Entry<Kind>>,
}

impl<Kind> Tree<Kind> {
    pub const fn root(&self) -> Node<Kind> {
        Node {
            index: 0,
            tree: self,
        }
    }

    pub fn children(&self, parent: Index) -> impl Iterator<Item = Index> {
        let mut next = parent + 1;
        let end = parent + self.entries[usize(parent)].size;
        core::iter::from_fn(move || {
            if next == end {
                return None;
            }
            let child = next;
            next += self.entries[usize(next)].size;
            Some(child)
        })
    }
}

struct Entry<Kind> {
    kind: Kind,
    span: Span,
    size: Index,
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
        self.tree.entries[usize(self.index)].kind
    }

    pub fn span(self) -> Span {
        self.tree.entries[usize(self.index)].span
    }

    pub fn children(self) -> impl Iterator<Item = Self> {
        let tree = self.tree;
        tree.children(self.index).map(|index| Self { index, tree })
    }
}

pub struct Token<Kind> {
    pub kind: Kind,
    pub span: Span,
}

pub struct Builder<'tokens, Kind> {
    tree: Tree<Kind>,
    tokens: core::slice::Iter<'tokens, Token<Kind>>,
    stack: Vec<usize>,
}

impl<'tokens, Kind> Builder<'tokens, Kind> {
    pub fn new(tokens: &'tokens [Token<Kind>]) -> Self {
        let entries = Vec::new();
        Self {
            tree: Tree { entries },
            tokens: tokens.iter(),
            stack: Vec::new(),
        }
    }

    /// The span is expanded as child nodes are added.
    ///
    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn start_node(&mut self, kind: Kind, initial_span: Span) {
        assert!(self.tree.entries.len() < usize(Index::MAX) - 1);
        self.stack.push(self.tree.entries.len());
        self.tree.entries.push(Entry {
            kind,
            span: initial_span,
            size: 0, // Dummy value which gets replaced when the node is closed.
        });
    }

    /// # Panics
    ///
    /// Panics if no node has been started.
    pub fn finish_node(&mut self) {
        let entry = self.stack.pop().unwrap();
        self.tree.entries[entry].size = (self.tree.entries.len() - entry).try_into().unwrap();
    }

    pub fn checkpoint(&self, initial_span: Span) -> Checkpoint {
        #[expect(
            clippy::missing_panics_doc,
            reason = "length is checked when pushing entries"
        )]
        Checkpoint {
            index: self.tree.entries.len().try_into().unwrap(),
            initial_span,
        }
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn finish_node_at(&mut self, checkpoint: Checkpoint, kind: Kind) {
        assert!(self.tree.entries.len() < usize(Index::MAX) - 1);
        self.tree.entries.insert(
            usize(checkpoint.index),
            Entry {
                kind,
                span: checkpoint.initial_span,
                size: Index::try_from(self.tree.entries.len()).unwrap() - checkpoint.index + 1,
            },
        );
        let mut span = checkpoint.initial_span;
        for child in self.tree.children(checkpoint.index) {
            span = span.merge(self.tree.entries[usize(child)].span);
        }
        self.tree.entries[usize(checkpoint.index)].span = span;
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn token(&mut self)
    where
        Kind: Copy,
    {
        assert!(self.tree.entries.len() < usize(Index::MAX) - 1);
        let Token { kind, span } = *self.tokens.next().unwrap();
        let node_span = &mut self.tree.entries[*self.stack.last().unwrap()].span;
        *node_span = node_span.merge(span);
        let size = 1;
        self.tree.entries.push(Entry { kind, span, size });
    }

    /// # Panics
    ///
    /// Panics if there are unfinished nodes.
    pub fn build(self) -> Tree<Kind> {
        assert!(self.stack.is_empty());
        let entries = self.tree.entries;
        Tree { entries }
    }
}

#[derive(Clone, Copy)]
pub struct Checkpoint {
    index: Index,
    initial_span: Span,
}

type Index = u32;

const fn usize(index: Index) -> usize {
    const {
        assert!(size_of::<Index>() <= size_of::<usize>());
    }
    index as usize
}

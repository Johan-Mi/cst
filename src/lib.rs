#![no_std]
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
}

struct Entry<Kind> {
    kind: Kind,
    span: Span,
    end: Index,
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
        let mut next = self.index + 1;
        let end = tree.entries[usize(self.index)].end;
        core::iter::from_fn(move || {
            if next == end {
                return None;
            }
            let index = next;
            next = tree.entries[usize(next)].end;
            Some(Self { index, tree })
        })
    }
}

pub struct Token<Kind> {
    pub kind: Kind,
    pub span: Span,
}

pub struct Builder<'tokens, Kind> {
    events: Vec<Event<Kind>>,
    tokens: core::slice::Iter<'tokens, Token<Kind>>,
    stack: Vec<Index>,
}

impl<'tokens, Kind> Builder<'tokens, Kind> {
    pub fn new(tokens: &'tokens [Token<Kind>]) -> Self {
        Self {
            events: Vec::new(),
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
        assert!(self.events.len() < usize(Index::MAX));
        self.events.push(Event::Open { kind, initial_span });
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn finish_node(&mut self) {
        assert!(self.events.len() < usize(Index::MAX));
        self.events.push(Event::Close);
    }

    pub fn checkpoint(&self, initial_span: Span) -> Checkpoint {
        #[expect(
            clippy::missing_panics_doc,
            reason = "length is checked when pushing events"
        )]
        Checkpoint {
            index: self.events.len().try_into().unwrap(),
            initial_span,
        }
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn finish_node_at(&mut self, checkpoint: Checkpoint, kind: Kind) {
        assert!(self.events.len() < usize(Index::MAX) - 1);
        self.events.insert(
            usize(checkpoint.index),
            Event::Open {
                kind,
                initial_span: checkpoint.initial_span,
            },
        );
        self.events.push(Event::Close);
    }

    /// # Panics
    ///
    /// Panics if the tree is too large.
    pub fn token(&mut self) {
        assert!(self.events.len() < usize(Index::MAX));
        self.events.push(Event::Token);
    }

    /// # Panics
    ///
    /// Panics if the started and finished nodes do not match.
    pub fn build(mut self) -> Tree<Kind>
    where
        Kind: Copy,
    {
        let mut entries = Vec::new();

        for event in self.events {
            match event {
                Event::Open { kind, initial_span } => {
                    self.stack.push(entries.len().try_into().unwrap());
                    entries.push(Entry {
                        kind,
                        span: initial_span,
                        end: 0, // Dummy value which gets replaced when the node is closed.
                    });
                }
                Event::Token => {
                    let Token { kind, span } = *self.tokens.next().unwrap();
                    let node_span = &mut entries[usize(*self.stack.last().unwrap())].span;
                    *node_span = node_span.merge(span);
                    entries.push(Entry {
                        kind,
                        span,
                        end: (entries.len() + 1).try_into().unwrap(),
                    });
                }
                Event::Close => {
                    entries[usize(self.stack.pop().unwrap())].end =
                        entries.len().try_into().unwrap();
                }
            }
        }

        Tree { entries }
    }
}

#[derive(Clone, Copy)]
pub struct Checkpoint {
    index: Index,
    initial_span: Span,
}

enum Event<Kind> {
    Open { kind: Kind, initial_span: Span },
    Token,
    Close,
}

type Index = u32;

const fn usize(index: Index) -> usize {
    const {
        assert!(size_of::<Index>() <= size_of::<usize>());
    }
    index as usize
}

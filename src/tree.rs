//! Implementation of the [suffix tree](https://web.stanford.edu/~mjkay/gusfield.pdf)
//! construction of which is performed in linear time

//! # Examples
//!
//! ```
//!     use suff_collections::{array::*, tree::*, lcp::*};
//!
//!     // let word = "Some word";
//!     let word: &str = "Some word\0";
//!     let find: &str = "word";
//!
//!     // construct suffix tree
//!     let st: SuffixTree = SuffixTree::new(word);
//!
//!     // finds the entry position of the line 'find' in 'word'
//!     let res: Option<usize> = st.find(find);
//!
//!     // construct lcp
//!     // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
//!     // let lcp: LCP = st.lcp_stack();
//!     let lcp: LCP = st.lcp_rec();
//!
//!     // convert suffix tree to suffix array
//!     // let sa: SuffixArray = SuffixArray::from_stack(st);
//!     let sa: SuffixArray = SuffixArray::from_rec(st);
//! ```

// TODO: maybe migration to DOP (suffix tree is struct of array)
use std::collections::BTreeMap;

use alloc::{vec::Vec, borrow::Cow};
use core::{str, option::Option};

use crate::{array::*, lcp::*, canonic_word};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct NodeIdx(usize);
impl NodeIdx {
    /// Return node index
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let root_node: &Node = st.node(NodeIdx::root());
    /// assert_eq!(st.root_node(), root_node);
    ///```
    pub fn root() -> Self {
        NodeIdx(0)
    }
    // allowed only in this module
    pub(self) fn new(n: usize) -> Self {
        NodeIdx(n)
    }

    /// Return number node index
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let root_idx: NodeIdx = NodeIdx::root();
    /// assert_eq!(root_idx.unwrap(), 0);
    ///```
    pub fn unwrap(self) -> usize {
        self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Node {
    link: Option<NodeIdx>,
    parent: NodeIdx,
    children: BTreeMap<u8, NodeIdx>,
    len: usize,
    pos: usize,
}

impl Node {
     /// Return suffix link
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let tree = SuffixTree::new("word");
    /// let link_idx = tree.node(NodeIdx::root()).link().unwrap();
    /// assert_eq!(link_idx, NodeIdx::root());
    ///```
    #[inline]
    pub fn link(&self) -> Option<NodeIdx> {
        self.link
    }

    /// Return parent index
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let tree = SuffixTree::new("word");
    /// let node_idx = tree.try_to_node(NodeIdx::root(), 'r' as u8).unwrap();
    /// let node_idx = tree.node(node_idx).parent();
    /// assert_eq!(node_idx, NodeIdx::root());
    ///```
    #[inline]
    pub fn parent(&self) -> NodeIdx {
        self.parent
    }

    /// Return ref on children
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let tree = SuffixTree::new("word");
    /// let children_num = tree.node(NodeIdx::root()).children().iter().count();
    /// assert_eq!(children_num, "word\0".len());
    ///```
    #[inline]
    pub fn children(&self) -> &BTreeMap<u8, NodeIdx> {
        &self.children
    }

    /// Return edge length in tree
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let tree = SuffixTree::new("word");
    /// let node_idx = tree.try_to_node(NodeIdx::root(), 'r' as u8).unwrap();
    /// let node = tree.node(node_idx);
    /// assert_eq!(node.len(), 3);
    ///```
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Return start pos in tree
    ///```
    /// use suff_collections::tree::*;
    ///
    /// let tree = SuffixTree::new("word");
    /// let node_idx = tree.try_to_node(NodeIdx::root(), 'r' as u8).unwrap();
    /// let node = tree.node(node_idx);
    /// assert_eq!(node.pos(), 2);
    ///```
    #[inline]
    pub fn pos(&self) -> usize {
        self.pos
    }
}

#[derive(Debug)]
struct State {
    node_idx: NodeIdx,
    edge_pos: usize,
}

#[derive(Clone)]
pub struct SuffixTree<'t> {
    word: Cow<'t, str>,
    v: Vec<Node>,
}

impl<'t> SuffixTree<'t> {

    /// Construct suffix tree. Complexity O(n)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st: SuffixTree = SuffixTree::new("word");
    /// ```
    /// At the end of the line should hit '\0'.
    /// If there is no '\0' at the end then the line will be copied and added '\0' to the end.
    /// Otherwise, the value will be taken by reference
    pub fn new(word: &'t str) -> Self {
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                v: vec![Node {
                    link: Some(NodeIdx::root()),
                    parent: NodeIdx::root(),
                    children: BTreeMap::new(),
                    len: 0,
                    pos: 0,
                }],
            };
        }
        let new_word = canonic_word(word);
        let mut tree = Self {
            word: new_word,
            v: vec![Node {
                link: Some(NodeIdx::root()),
                parent: NodeIdx::root(),
                children: BTreeMap::new(),
                len: 0,
                pos: 0,
            }],
        };
        tree.build_ukkonen();
        tree.shrink_to_fit();
        tree
    }

    /// Construct suffix tree from suffix array. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// let st: SuffixArray = SuffixArray::new("word\0");
    /// let sa: SuffixTree = SuffixTree::from(st);
    /// ```
    pub fn from(array: SuffixArray) -> Self {
        let lcp = array.lcp();
        let (word, sa) = array.split_owned();
        let mut tree = Self {
            word: match word {
                Cow::Borrowed(x) => Cow::from(x.to_owned()),
                Cow::Owned(x) => Cow::from(x),
            },
            v: vec![Node {
                link: Some(NodeIdx::root()),
                parent: NodeIdx::root(),
                children: BTreeMap::new(),
                len: 0,
                pos: 0,
            }],
        };

        let tree_size = tree.max_tree_size();
        tree.reserve(tree_size);
        let mut total_len = vec![0; tree_size];
        let mut node_idx = NodeIdx::root();
        let word = tree.word.as_bytes().to_owned();

        for (i, &start) in sa.iter().enumerate() {
            // safe because sa is correct suffix array
            let pref_len = unsafe { *lcp.idx(i) };

            loop {
                // safe because 0 <= node_idx < tree.len() && total_len.len() == tree.len()
                if pref_len == unsafe { *total_len.get_unchecked(node_idx.unwrap()) }
                    || tree.is_root(node_idx)
                {
                    let start = start + pref_len;
                    let len = word.len() - start;
                    let add_idx = tree.add_node(node_idx, len, start);

                    // safe because 0 <= start < word.len() && pref is lcp
                    let ch = unsafe { *word.get_unchecked(start) };
                    tree.set_child(node_idx, add_idx, ch);

                    // safe because 0 <= node_idx, add_idx < tree.len() && total_len.len() == tree.len()
                    unsafe {
                        *total_len.get_unchecked_mut(add_idx.unwrap()) =
                            total_len.get_unchecked(node_idx.unwrap()) + len;
                    }
                    node_idx = add_idx;
                    break;

                // safe because 0 <= node_idx, parent_idx < tree.len() && total_len.len() == tree.len()
                } else if unsafe { pref_len < *total_len.get_unchecked(node_idx.unwrap())
                    && pref_len > *total_len.get_unchecked(tree.node(node_idx).parent.unwrap()) }
                {
                    let start = start + pref_len;
                    let len = word.len() - start;

                    // safe because 0 <= node_idx < tree.len() && total_len.len() == tree.len()
                    let suf_len = unsafe { *total_len.get_unchecked(node_idx.unwrap()) } - pref_len;
                    let pref_len = tree.node(node_idx).len - suf_len;
                    let split_idx = tree.split(
                        &word,
                        node_idx,
                        tree.node(node_idx).pos + pref_len,
                        pref_len,
                        suf_len
                    );

                    // safe because 0 <= node_idx, split_idx < tree.len() && total_len.len() == tree.len()
                    unsafe {
                        *total_len.get_unchecked_mut(split_idx.unwrap()) =
                            *total_len.get_unchecked(node_idx.unwrap()) - suf_len;
                    }
                    let add_idx = tree.add_node(split_idx, len, start);

                    // safe because 0 <= start < word.len() && pref is lcp
                    let ch = unsafe { *word.get_unchecked(start) };
                    tree.set_child(split_idx, add_idx, ch);

                    // safe because 0 <= node_idx, split_idx < tree.len() && total_len.len() == tree.len()
                    unsafe {
                        *total_len.get_unchecked_mut(add_idx.unwrap()) =
                            *total_len.get_unchecked(split_idx.unwrap()) + len;
                    }
                    node_idx = add_idx;
                    break;
                } else {
                    node_idx = tree.node(node_idx).parent;
                }
            }
        }

        tree.shrink_to_fit();
        // correct because tree is correct suffix tree && array.word().last() == '\0'
        tree
    }

    /// Find substr. Complexity O(|word|)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let find: Option<usize> = st.find("or");
    /// assert_eq!(find, Some(1));
    /// ```
    pub fn find(&self, find: &str) -> Option<usize> {
        let (word, find) = (self.word.as_bytes(), find.as_bytes());
        let mut node_idx = NodeIdx::root();
        let mut curr_pos = 0;
        loop {
            let (end_edge_pos, edge_pos) = self.edge_propagate(&mut node_idx, &mut curr_pos, word, find)?;
            if curr_pos == find.len() {
                return Some(edge_pos - find.len());
            }
            if edge_pos != end_edge_pos {
                return None;
            }
        }
    }

    /// lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
    /// Construct LCP not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let lcp = SuffixTree::new("word").lcp_stack();
    /// ```
    pub fn lcp_stack(&self) -> LCP {
        let mut lcp = Vec::with_capacity(self.word.len());
        lcp.push(0);
        let mut stack = Vec::with_capacity(self.word.len());
        let mut prev_len = 0;

        stack.push(LCPChildrenIterator {
            it: self.node(NodeIdx::root()).children.iter(),
            len: 0,
            node_len: 0,
        });
        stack.last_mut().unwrap().it.next();
        while let Some(x) = stack.last_mut() {
            match x.it.next() {
                None => {
                    prev_len -= x.node_len;
                    stack.pop();
                }
                Some((_, &i)) => {
                    let node = self.node(i);
                    if node.children.is_empty() {
                        lcp.push(prev_len);
                        prev_len = x.len;
                    } else {
                        let len = x.len + node.len;
                        stack.push(LCPChildrenIterator {
                            it: node.children.iter(),
                            len: len,
                            node_len: node.len,
                        });
                    }
                }
            }
        }
        lcp.shrink_to_fit();
        return LCP::new(lcp);

        struct LCPChildrenIterator<I>
        where
            I: Iterator,
        {
            it: I,
            len: usize,
            node_len: usize,
        }
    }

    /// lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
    /// Construct LCP recursive. Complexity O(n)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let lcp = SuffixTree::new("word").lcp_rec();
    /// ```
    pub fn lcp_rec(&self) -> LCP {
        let node = self.node(NodeIdx::root());
        if node.children.is_empty() {
            return LCP::new(vec![]);
        }
        let mut lcp = Vec::with_capacity(self.word.len());
        lcp.push(0);
        let mut prev_len = 0;
        for (_, &child) in node.children.iter().skip(1) {
            self.lcp_rec_inner(child, node.len, &mut prev_len, &mut lcp);
        }
        lcp.shrink_to_fit();
        LCP::new(lcp)
    }

    /// if word is ascii then print data in .dot format for graphviz
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// SuffixTree::new("word").to_graphviz();
    /// ```
    pub fn to_graphviz(&self) {
        assert!(self.word.is_ascii());
        println!("digraph G {{");
        let size = (self.v.len() as f64 * 0.5) as u64;
        println!("    size=\"{}, {}\"", size, size);
        for (i, x) in self.v.iter().enumerate() {
            let node_name = &self.word[x.pos..x.pos + x.len];
            let child = &x.children;
            for (_, &node_idx) in child {
                let start = self.node(node_idx).pos;
                let end = start + self.node(node_idx).len;
                let label_name = self.word.as_bytes()[start] as char;
                let children_name = &self.word[start..end];
                println!("    _{}_{} -> _{}_{} [byte_label=\"_{}_{}\\npos: {}, len: {}\"]",
                i, node_name, node_idx.unwrap(), children_name, node_idx.unwrap(), label_name, start, end - start);
            }
            match x.link {
                Some(link_idx) => {
                    let start = self.node(link_idx).pos;
                    let end = start + self.node(link_idx).len;
                    let link_name = &self.word[start..end];
                    println!("    _{}_{} -> _{}_{} [style=dotted]", i, node_name, link_idx.unwrap(), link_name);
                }
                None => (),
            }
        }
        println!("}}");
    }

    /// Semantic to check if the node is root
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let root_idx: NodeIdx = NodeIdx::root();
    /// let is_root: bool = SuffixTree::new("word").is_root(root_idx);
    /// assert_eq!(is_root, true);
    /// ```
    #[inline]
    pub fn is_root(&self, node_idx: NodeIdx) -> bool {
        node_idx == NodeIdx::root()
    }

    /// Return ref on word
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let word: &str = st.word();
    /// assert_eq!(word, "word\0");
    /// ```
    #[inline]
    pub fn word(&self) -> &str {
        &self.word
    }

    /// Return ref on node in suffix tree
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let node: &Node = st.node(NodeIdx::root());
    /// ```
    #[inline]
    pub fn node(&self, node_idx: NodeIdx) -> &Node {
        // safe because 0 <= node_idx < self.v.len():
        //      node_idx is something by the type of a pointer to memory
        //      that cannot be NULL (None) and to everything is in
        //      the range (0..self.v().len()] because the generation
        //      of new values in the NodeIdx type is provided only by the
        //      NodeIdx::new(n) method, but this method can be used only in this module
        //      and the NodeIdx::new(n) method is called only when adding a new Node.
        //      The tree always contains the root node
        unsafe { self.v.get_unchecked(node_idx.unwrap()) }
    }

    /// Return ref on root node in suffix tree
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let root_node: &Node = st.root_node();
    /// assert_eq!(root_node, st.node(NodeIdx::root()));
    /// ```
    #[inline]
    pub fn root_node(&self) -> &Node {
        // safe because suffix tree always have self.v.len() >= 1
        unsafe { self.v.get_unchecked(0) }
    }

    /// Go to the next node.
    /// If there is no transition then return None
    /// else return node index
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let word = "word";
    /// let st = SuffixTree::new(&word);
    /// let node: Option<NodeIdx> = st.try_to_node(NodeIdx::root(), 'w' as u8);
    /// let none_node: Option<NodeIdx> = st.try_to_node(NodeIdx::root(), 'p' as u8);
    /// // assert_eq!(none_node, None);
    ///
    /// // walk example
    /// let tree = SuffixTree::new(&word);
    /// let mut node_idx = NodeIdx::root();
    /// let mut symb = *word.as_bytes().first().unwrap();
    /// loop {
    ///     node_idx = tree.try_to_node(node_idx, symb).unwrap();
    ///     let node = tree.node(node_idx);
    ///     let idx = node.pos() + node.len();
    ///     println!("start symbol byte: {}", symb);
    ///     println!("pos...len: {}...{}", node.pos(), node.len());
    ///     if idx < word.len() {
    ///         symb = word.as_bytes()[idx];
    ///     } else {
    ///         break;
    ///     }
    /// }
    /// ```
    #[inline]
    pub fn try_to_node(&self, current_node: NodeIdx, transition: u8) -> Option<NodeIdx> {
        match self.node(current_node).children.get(&transition) {
            Some(&x) => Some(x),
            None => None,
        }
    }

    // leaf size == word.len()
    // inner node <= word.len() because we do word.len() operation (insert and add node) or add node
    // |tree| == leaf size + inner node <= 2 * |word|
    #[inline]
    fn max_tree_size(&self) -> usize {
        2 * self.word.len()
    }
    #[inline]
    fn shrink_to_fit(&mut self) {
        self.v.shrink_to_fit();
    }
    #[inline]
    fn build_ukkonen(&mut self) {
        self.reserve(self.max_tree_size());
        let mut s = State {
            node_idx: NodeIdx::root(),
            edge_pos: 0,
        };
        let word = self.word.as_bytes().to_owned();
        for (i, &ch) in word.iter().enumerate() {
            while
                self
                .try_transfer_to(&mut s, ch)
                .if_not_transfer(|x| x.create_transfer(&word, i).to_link(&word))
            {}
        }
    }

    #[inline]
    fn try_transfer_to<'r, 's>(&'r mut self, s: &'s mut State, ch: u8) -> Transfer<'r, 't, 's> {
        if self.is_end_edge(&s) {
            if self.node(s.node_idx).children.contains_key(&ch) {
                s.node_idx = self.to_node(s.node_idx, ch);
                s.edge_pos = self.node(s.node_idx).pos + 1;
                return Transfer::Succes;
            } else {
                return Transfer::StopInNode(self, s, ch);
            }
        }
        // safe because s.edge_pos < self.word.len()
        // if s.edge_pos >= self.word.len() then algorithm is done
        if unsafe { *self.word.as_bytes().get_unchecked(s.edge_pos) } == ch {
            s.edge_pos += 1;
            return Transfer::Succes;
        } else {
            return Transfer::StopInEdge(self, s, ch);
        }
    }
    #[inline]
    fn update_link(&mut self, s: &mut State, link: NodeIdx) {
        self.node_mut(s.node_idx).link = Some(link);
        let node = self.node(link);
        s.node_idx = link;
        s.edge_pos = node.pos + node.len;
    }
    #[inline]
    fn split(&mut self, word: &[u8], node_idx: NodeIdx, split_word_pos: usize, pref_len: usize, suf_len: usize) -> NodeIdx {
        let insert_node = self.add_node(self.node(node_idx).parent, pref_len, self.node(node_idx).pos);
        self.set_child(
            insert_node,
            node_idx,
            // safe because split_word_pos < self.word.len()
            // if split_word_pos >= self.word.len() then never call insert because processing self.word is done
            unsafe { *word.get_unchecked(split_word_pos) }
        );

        self.set_child(
            self.node(node_idx).parent,
            insert_node,
            // safe because self.node(node_idx).pos < self.word.len()
            // if self.node(node_idx).pos >= self.word.len() never hepens because in tree all self.node.pos < self.word.len()
            unsafe { *word.get_unchecked(self.node(node_idx).pos) }
        );

        self.update_edge(node_idx, insert_node, suf_len, split_word_pos);
        insert_node
    }
    #[inline]
    fn skip_walk(&self, word: &[u8], link: &mut NodeIdx, word_pos: &mut usize, edge_len: &mut usize) {
        loop {
            // safe because word_pos < self.word.len().
            // if self.node(s.node_idx).pos >= self.word.len() never hepens because in tree all self.node.pos < self.word.len()
            let key = unsafe { *word.get_unchecked(*word_pos) };
            *link = self.to_node(*link, key);
            let node = self.node(*link);
            if *edge_len > node.len {
                *edge_len -= node.len;
                *word_pos += node.len;
            } else {
                break;
            }
        }
    }

    #[inline]
    fn reserve(&mut self, size: usize) {
        self.v.reserve(size);
    }
    #[inline]
    fn is_end_edge(&self, s: &State) -> bool {
        let node = self.node(s.node_idx);
        s.edge_pos == node.pos + node.len
    }
    #[inline]
    fn generate_node_idx(&self) -> NodeIdx {
        NodeIdx::new(self.v.len())
    }
    #[inline]
    fn node_mut(&mut self, node_idx: NodeIdx) -> &mut Node {
        unsafe { self.v.get_unchecked_mut(node_idx.unwrap()) }
    }

    #[inline]
    fn to_node(&self, current_node: NodeIdx, transition: u8) -> NodeIdx {
        *self.node(current_node).children.get(&transition).unwrap()
    }
    #[inline]
    fn set_child(&mut self, node_idx: NodeIdx, child_idx: NodeIdx, ch: u8) {
        self.node_mut(node_idx).children.insert(ch, child_idx);
    }
    #[inline]
    fn add_node(&mut self, parent: NodeIdx, len: usize, pos: usize) -> NodeIdx {
        let add_node_idx = self.generate_node_idx();
        self.v.push(Node {
            link: None,
            parent: parent,
            children: BTreeMap::new(),
            len: len,
            pos: pos,
        });
        add_node_idx
    }
    #[inline]
    fn update_edge(&mut self, node_idx: NodeIdx, parent: NodeIdx, len: usize, pos: usize) {
        let node = self.node_mut(node_idx);
        node.parent = parent;
        node.len = len;
        node.pos = pos;
    }
    #[inline]
    fn lcp_rec_inner(&self, node_idx: NodeIdx, len: usize, prev_len: &mut usize, lcp: &mut Vec<usize>) {
        let node = self.node(node_idx);
        if node.children.is_empty() {
            lcp.push(*prev_len);
            *prev_len = len;
            return;
        }
        for (_, &child) in &node.children {
            self.lcp_rec_inner(child, len + node.len, prev_len, lcp);
        }
        *prev_len -= node.len;
    }
    #[inline]
    fn edge_propagate(&self, node_idx: &mut NodeIdx, curr_pos: &mut usize, word: &[u8], find: &[u8]) -> Option<(usize, usize)> {
        // safe because curr_pos < self.word.len() && node_idx < self.v.len()
        // if self.node(s.node_idx).pos >= self.word.len() never hepens because in tree all self.node.pos < self.word.len()
        *node_idx = unsafe { self.try_to_node(*node_idx, *find.get_unchecked(*curr_pos))? };
        let node = self.node(*node_idx);
        let mut edge_pos = node.pos;
        let end_edge_pos = node.pos + node.len;
        while edge_pos < end_edge_pos && *curr_pos < find.len()
        // safe by previous check
            && unsafe { *find.get_unchecked(*curr_pos) == *word.get_unchecked(edge_pos) }
        {
            edge_pos += 1;
            *curr_pos += 1;
        } 
        Some((end_edge_pos, edge_pos))
    }
}

enum Transfer<'r, 't, 's> {
    StopInEdge(&'r mut SuffixTree<'t>, &'s mut State, u8),
    StopInNode(&'r mut SuffixTree<'t>, &'s mut State, u8),
    Succes,
}

impl<'r, 't, 's> Transfer<'r, 't, 's> {
    #[inline]
    fn if_not_transfer<F>(self, f: F) -> bool
    where
        F: Fn(Transfer) -> bool,
    {
        match self {
            Transfer::Succes => false,
            Transfer::StopInEdge(_, _, _)
            | Transfer::StopInNode(_, _, _) => f(self),
        }
    }
    #[inline]
    fn create_transfer(self, word: &[u8], ch_pos: usize) -> Transfer<'r, 't, 's> {
        match self {
            Transfer::StopInEdge(tree, s, ch) => {
                let suf_len = tree.node(s.node_idx).pos + tree.node(s.node_idx).len - s.edge_pos;
                let pref_len = tree.node(s.node_idx).len - suf_len;
                s.node_idx = tree.split(word, s.node_idx, s.edge_pos, pref_len, suf_len);
                let add_node = tree.add_node(s.node_idx, tree.word.len() - ch_pos, ch_pos);
                tree.set_child(s.node_idx, add_node, ch);
                Transfer::StopInNode(tree, s, ch)
            }
            Transfer::StopInNode(tree, s, ch) => {
                let add_node = tree.add_node(s.node_idx, tree.word.len() - ch_pos, ch_pos);
                tree.set_child(s.node_idx, add_node, ch);
                Transfer::StopInNode(tree, s, ch)
            }
            Transfer::Succes => unreachable!(),
        }
    }
    #[inline]
    fn to_link(self, word: &[u8]) -> bool {
        match self {
            Transfer::StopInNode(tree, s, _) => {
                if tree.is_root(s.node_idx) {
                    return false;
                }
                let node = tree.node_mut(s.node_idx);
                match node.link {
                    Some(link) => {
                        s.node_idx = link;
                        s.edge_pos = tree.node(link).pos + tree.node(link).len;
                    }
                    None => {
                        let mut edge_len = node.len;
                        let mut word_pos = node.pos;
                        let parent_idx = node.parent;
                        if parent_idx == NodeIdx::root() {
                            if node.len == 1 {
                                node.link = Some(NodeIdx::root());
                                s.node_idx = NodeIdx::root();
                                s.edge_pos = 0;
                                return true;
                            } else {
                                edge_len -= 1;
                                word_pos += 1;
                            }
                        }
                        let mut link = tree.node(parent_idx).link.unwrap();
                        tree.skip_walk(word, &mut link, &mut word_pos, &mut edge_len);
                        if edge_len == tree.node(link).len {
                            tree.update_link(s, link);
                        } else {
                            word_pos += edge_len;
                            let s_link = State {
                                node_idx: link,
                                edge_pos: word_pos,
                            };
                            let insert_node =
                                tree.split(
                                    word,
                                    s_link.node_idx,
                                    s_link.edge_pos,
                                    edge_len,
                                    tree.node(link).len - edge_len
                                );
                            tree.update_link(s, insert_node);
                        }
                    }
                };
                true
            }
            Transfer::StopInEdge(_, _, _)
            | Transfer::Succes => unreachable!(),
        }
    }
}
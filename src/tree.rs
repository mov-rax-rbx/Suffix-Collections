//! Implementation of the [suffix tree](https://web.stanford.edu/~mjkay/gusfield.pdf)
//! construction of which is performed in linear time

//! # Examples
//!
//! ```
//! use suff_collections::{array::*, tree::*, lcp::*};
//!
//! let word: &str = "Some word";
//! let find: &str = "word";
//!
//! // construct suffix tree
//! let st: SuffixTree = SuffixTree::new(word);
//!
//! // finds the entry position of the line 'find' in 'word'
//! let res: Option<usize> = st.find(find);
//!
//! // construct lcp
//! // lcp[i] = max_pref(sa[i], sa[i - 1]) && lcp.len() == sa.len()
//! // let lcp: LCP<u8> = st.lcp::<u8>();
//! // let lcp: LCP<u16> = st.lcp::<u16>();
//! // let lcp: LCP<u32> = st.lcp::<u32>();
//! let lcp: LCP<usize> = st.lcp::<usize>();
//!
//! // convert suffix tree to suffix array
//! // let sa = SuffixArray::<u8>::from(st);
//! // let sa = SuffixArray::<u16>::from(st);
//! // let sa = SuffixArray::<u32>::from(st);
//! let sa = SuffixArray::<usize>::from(st);
//!
//! // construct online suffix tree
//! let mut ost: OnlineSuffixTree = OnlineSuffixTree::new();
//!
//! // add word to online suffix tree
//! ost.add(word);
//!
//! // finds the entry position of the line 'find' in 'word'
//! let res: Option<usize> = ost.find(find);
//!
//! // conver online suffix tree to suffix tree
//! let st = ost.finish();
//! ```

#![forbid(unsafe_code)]

use alloc::collections::BTreeMap;
use alloc::{borrow::Cow, borrow::ToOwned, string::String, vec::Vec};
use core::{fmt::Write, format_args, option::Option, str};

use crate::{array::build_suffix_array::SuffixIndices, array::*, lcp::*};

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
    /// assert_eq!(children_num, "word".len() + 1);
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
    /// assert_eq!(node.len(), 2);
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

#[derive(Debug, Clone)]
struct State {
    node_idx: NodeIdx,
    edge_pos: usize,
}

#[derive(Debug, Clone)]
pub struct SuffixTree<'t> {
    word: Cow<'t, str>,
    tree: AloneSuffixTree,
}

impl<'t> SuffixTree<'t> {
    /// Construct suffix tree. Complexity O(n)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st: SuffixTree = SuffixTree::new("word");
    /// ```
    pub fn new(word: &'t str) -> Self {
        if word.is_empty() {
            return Self {
                word: Cow::from(""),
                tree: AloneSuffixTree {
                    nodes: vec![Node {
                        link: Some(NodeIdx::root()),
                        parent: NodeIdx::root(),
                        children: BTreeMap::new(),
                        len: 0,
                        pos: 0,
                    }],
                },
            };
        }
        let mut tree = Self {
            word: Cow::from(word),
            tree: AloneSuffixTree {
                nodes: vec![Node {
                    link: Some(NodeIdx::root()),
                    parent: NodeIdx::root(),
                    children: BTreeMap::new(),
                    len: 0,
                    pos: 0,
                }],
            },
        };
        tree.build_ukkonen();
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
        self.tree.find(&self.word, find, false)
    }

    /// lcp\[i\] = max_pref(sa\[i\], sa\[i - 1]\) && lcp.len() == sa.len()
    /// Construct LCP not recursive. Complexity O(n)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// // let lcp = SuffixTree::new("word").lcp::<u8>();
    /// // let lcp = SuffixTree::new("word").lcp::<u16>();
    /// // let lcp = SuffixTree::new("word").lcp::<u32>();
    /// let lcp = SuffixTree::new("word").lcp::<usize>();
    /// ```
    pub fn lcp<T: SuffixIndices<T>>(&self) -> LCP<T> {
        let mut lcp = Vec::<T>::with_capacity(self.word.len() + 1);
        let mut stack = Vec::with_capacity(self.word.len() + 1);
        let mut prev_len = 0;

        stack.push(LCPChildrenIterator {
            it: self.node(NodeIdx::root()).children.iter(),
            len: 0,
            node_len: 0,
        });
        while let Some(x) = stack.last_mut() {
            match x.it.next() {
                None => {
                    prev_len -= x.node_len;
                    stack.pop();
                }
                Some((_, &i)) => {
                    let node = self.node(i);
                    if node.children.is_empty() {
                        lcp.push(T::try_from(prev_len).ok().unwrap());
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

    /// if word is ascii then print data in .dot format for graphviz
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let buff = SuffixTree::new("word").to_graphviz();
    /// println!("{}", buff);
    /// ```
    pub fn to_graphviz(&self) -> String {
        assert!(self.word.is_ascii());

        let mut f = String::new();
        let size = (self.tree.nodes.len() as f64 * 0.5) as u64;
        f.write_str("digraph G {\n").unwrap();
        f.write_fmt(format_args!("    size=\"{}, {}\"\n", size, size))
            .unwrap();
        for (i, x) in self.tree.nodes.iter().enumerate() {
            let node_name = &self.word[x.pos..x.pos + x.len];
            let child = &x.children;
            for &node_idx in child.values() {
                let start = self.node(node_idx).pos;
                let end = start + self.node(node_idx).len;

                let (label_name, children_name) = if start != end {
                    (self.word.as_bytes()[start] as char, &self.word[start..end])
                } else {
                    (0 as char, "\0")
                };

                f.write_fmt(format_args!(
                    "    _{}_{} -> _{}_{} [byte_label=\"_{}_{}\\npos: {}, len: {}\"]\n",
                    i,
                    node_name,
                    node_idx.unwrap(),
                    children_name,
                    node_idx.unwrap(),
                    label_name,
                    start,
                    end - start
                ))
                .unwrap();
            }
            if let Some(link_idx) = x.link {
                let start = self.node(link_idx).pos;
                let end = start + self.node(link_idx).len;
                let link_name = &self.word[start..end];
                f.write_fmt(format_args!(
                    "    _{}_{} -> _{}_{} [style=dotted]\n",
                    i,
                    node_name,
                    link_idx.unwrap(),
                    link_name
                ))
                .unwrap();
            }
        }
        f.write_str("}").unwrap();
        f
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
        self.tree.is_root(node_idx)
    }

    /// Return ref on word
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let st = SuffixTree::new("word");
    /// let word: &str = st.word();
    /// assert_eq!(word, "word");
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
        self.tree.node(node_idx)
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
        &self.tree.nodes[0]
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
        self.tree.try_to_node(current_node, transition)
    }

    // |word| + 1 == |leaf| leaf is word suffix + terminal leaf
    // |inner node| <= |word| + 1 because we do |word|
    //  operation (insert and add node) or add node + root node
    // |tree| == |leaf| + |inner node| <= 2 * |leaf| <= 2 * (|word| + 1)
    #[inline]
    fn max_tree_size(&self) -> usize {
        2 * (self.word.len() + 1)
    }
    #[inline]
    fn reserve(&mut self, size: usize) {
        self.tree.nodes.reserve(size);
    }
    #[inline]
    fn shrink_to_fit(&mut self) {
        self.tree.nodes.shrink_to_fit();
    }
    #[inline]
    fn build_ukkonen(&mut self) {
        self.reserve(self.max_tree_size());
        let mut s = State {
            node_idx: NodeIdx::root(),
            edge_pos: 0,
        };
        for (i, &ch) in self.word.as_bytes().iter().enumerate() {
            while self
                .tree
                .try_transfer_to(&self.word, &mut s, ch, false)
                .if_transfer_not_success(|x| x.create_transfer(i).to_link())
            {}
        }

        // Add terminal node. Position `i` is safe because
        // we dont do access like self.word.as_bytes()[i] in
        // create_transfer or try_transfer_to and use `i` like length

        let i = self.word.len();
        let ch = 0;
        while self
            .tree
            .try_transfer_to(&self.word, &mut s, ch, false)
            .if_transfer_not_success(|x| x.create_transfer(i).to_link())
        {}

        self.shrink_to_fit();
    }
}

impl<T: SuffixIndices<T>> From<SuffixArray<'_, T>> for SuffixTree<'_> {
    /// Construct suffix tree from suffix array. Complexity O(n)
    /// ```
    /// use suff_collections::{array::*, tree::*};
    ///
    /// // let st = SuffixArray::<u8>::new("word");
    /// // let st = SuffixArray::<u16>::new("word");
    /// // let st = SuffixArray::<u32>::new("word");
    /// let st = SuffixArray::<usize>::new("word");
    /// let sa = SuffixTree::from(st);
    /// ```
    fn from(array: SuffixArray<T>) -> Self {
        let lcp = array.lcp();
        let (word, sa) = array.split_owned();
        let mut suff_tree = Self {
            word: match word {
                Cow::Borrowed(x) => Cow::from(x.to_owned()),
                Cow::Owned(x) => Cow::from(x),
            },
            tree: AloneSuffixTree {
                nodes: vec![Node {
                    link: Some(NodeIdx::root()),
                    parent: NodeIdx::root(),
                    children: BTreeMap::new(),
                    len: 0,
                    pos: 0,
                }],
            },
        };

        let tree_size = suff_tree.max_tree_size();
        suff_tree.reserve(tree_size);
        let mut total_len = vec![T::zero(); tree_size];
        let mut node_idx = NodeIdx::root();

        for (&start, &pref_len) in sa.iter().zip(lcp.iter()) {
            loop {
                if pref_len == total_len[node_idx.unwrap()] || suff_tree.is_root(node_idx) {
                    let start = (start + pref_len).to_usize();
                    let len = suff_tree.word.as_bytes().len() - start;
                    let add_idx = suff_tree.tree.add_node(node_idx, len, start);

                    let ch = suff_tree.word.as_bytes()[start];
                    suff_tree.tree.set_child(node_idx, add_idx, ch);

                    total_len[add_idx.unwrap()] =
                        total_len[node_idx.unwrap()] + T::try_from(len).ok().unwrap();
                    node_idx = add_idx;
                    break;
                } else if pref_len < total_len[node_idx.unwrap()]
                    && pref_len > total_len[suff_tree.node(node_idx).parent.unwrap()] {

                    let start = (start + pref_len).to_usize();
                    let len = suff_tree.word.as_bytes().len() - start;

                    let suf_len = total_len[node_idx.unwrap()] - pref_len;
                    let pref_len = suff_tree.node(node_idx).len - suf_len.to_usize();
                    let split_idx = suff_tree.tree.split(
                        &suff_tree.word,
                        node_idx,
                        suff_tree.node(node_idx).pos + pref_len,
                        pref_len,
                        suf_len.to_usize(),
                    );

                    total_len[split_idx.unwrap()] = total_len[node_idx.unwrap()] - suf_len;
                    let add_idx = suff_tree.tree.add_node(split_idx, len, start);

                    let ch = suff_tree.word.as_bytes()[start];
                    suff_tree.tree.set_child(split_idx, add_idx, ch);

                    total_len[add_idx.unwrap()] =
                        total_len[split_idx.unwrap()] + T::try_from(len).ok().unwrap();
                    node_idx = add_idx;
                    break;
                } else {
                    node_idx = suff_tree.node(node_idx).parent;
                }
            }
        }

        suff_tree.shrink_to_fit();
        suff_tree
    }
}

#[derive(Debug, Clone)]
pub struct OnlineSuffixTree {
    word: String,
    tree: AloneSuffixTree,
    build_info: State,
}

impl OnlineSuffixTree {
    /// Create online suffix tree
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let ost: OnlineSuffixTree = OnlineSuffixTree::new();
    /// ```
    pub fn new() -> Self {
        Self {
            word: String::new(),
            tree: AloneSuffixTree {
                nodes: vec![Node {
                    link: Some(NodeIdx::root()),
                    parent: NodeIdx::root(),
                    children: BTreeMap::new(),
                    len: 0,
                    pos: 0,
                }],
            },
            build_info: State {
                node_idx: NodeIdx::root(),
                edge_pos: 0,
            },
        }
    }

    /// Add word to suffix tree
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let mut ost: OnlineSuffixTree = OnlineSuffixTree::new();
    /// ost.add("wo");
    /// ost.add("r");
    /// ost.add("d");
    /// ```
    pub fn add(&mut self, add_word: &str) {
        let mut i = self.word.len();
        self.word.push_str(add_word);
        let mut s = self.build_info.clone();

        for &ch in add_word.as_bytes().iter() {
            while self
                .tree
                .try_transfer_to(&self.word, &mut s, ch, true)
                .if_transfer_not_success(|x| x.create_transfer(i).to_link())
            {}
            i += 1;
        }

        self.build_info = s;
    }

    /// Transform online suffix tree to suffix tree
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let mut ost: OnlineSuffixTree = OnlineSuffixTree::new();
    /// ost.add("wo");
    /// ost.add("r");
    /// ost.add("d");
    ///
    /// let suffix_tree = ost.finish();
    /// ```
    pub fn finish<'t>(mut self) -> SuffixTree<'t> {
        let mut s = self.build_info.clone();
        let i = self.word.len();
        let ch = 0;
        while self
            .tree
            .try_transfer_to(&self.word, &mut s, ch, true)
            .if_transfer_not_success(|x| x.create_transfer(i).to_link())
        {}

        self.word.shrink_to_fit();
        self.tree.nodes.shrink_to_fit();

        self.tree
            .nodes
            .iter_mut()
            .filter(|x| x.len == usize::MAX)
            .for_each(|x| x.len = i - x.pos);

        SuffixTree {
            word: Cow::from(self.word),
            tree: self.tree,
        }
    }

    /// Find substr. Complexity O(|word|)
    /// ```
    /// use suff_collections::tree::*;
    ///
    /// let mut st = OnlineSuffixTree::new();
    /// st.add("word");
    /// let find: Option<usize> = st.find("or");
    /// assert_eq!(find, Some(1));
    /// ```
    pub fn find(&self, find: &str) -> Option<usize> {
        self.tree.find(&self.word, find, true)
    }
}

impl Default for OnlineSuffixTree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
#[repr(transparent)]
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
    pub(crate) fn new(n: usize) -> Self {
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

#[derive(Debug, Clone)]
#[repr(transparent)]
struct AloneSuffixTree {
    nodes: Vec<Node>,
}

impl AloneSuffixTree {
    #[inline]
    fn node(&self, node_idx: NodeIdx) -> &Node {
        &self.nodes[node_idx.unwrap()]
    }
    #[inline]
    fn node_mut(&mut self, node_idx: NodeIdx) -> &mut Node {
        &mut self.nodes[node_idx.unwrap()]
    }
    #[inline]
    fn try_to_node(&self, current_node: NodeIdx, transition: u8) -> Option<NodeIdx> {
        match self.node(current_node).children.get(&transition) {
            Some(&x) => Some(x),
            None => None,
        }
    }
    #[inline]
    fn to_node(&self, current_node: NodeIdx, transition: u8) -> NodeIdx {
        self.try_to_node(current_node, transition).unwrap()
    }
    #[inline]
    fn is_root(&self, node_idx: NodeIdx) -> bool {
        node_idx == NodeIdx::root()
    }
    #[inline]
    fn is_end_edge(&self, s: &State) -> bool {
        let node = self.node(s.node_idx);
        s.edge_pos == node.pos + node.len
    }
    #[inline]
    fn update_edge(&mut self, node_idx: NodeIdx, parent: NodeIdx, len: usize, pos: usize) {
        let node = self.node_mut(node_idx);
        node.parent = parent;
        node.len = len;
        node.pos = pos;
    }
    #[inline]
    fn update_link(&mut self, s: &mut State, link: NodeIdx) {
        self.node_mut(s.node_idx).link = Some(link);
        let node = self.node(link);
        s.node_idx = link;
        s.edge_pos = node.pos + node.len;
    }

    #[inline]
    fn set_child(&mut self, node_idx: NodeIdx, child_idx: NodeIdx, ch: u8) {
        self.node_mut(node_idx).children.insert(ch, child_idx);
    }
    #[inline]
    fn add_node(&mut self, parent: NodeIdx, len: usize, pos: usize) -> NodeIdx {
        let add_node_idx = NodeIdx::new(self.nodes.len());
        self.nodes.push(Node {
            link: None,
            parent: parent,
            children: BTreeMap::new(),
            len: len,
            pos: pos,
        });
        add_node_idx
    }
    #[inline]
    fn split(
        &mut self,
        word: &str,
        node_idx: NodeIdx,
        split_word_pos: usize,
        pref_len: usize,
        suf_len: usize,
    ) -> NodeIdx {
        let insert_node = self.add_node(
            self.node(node_idx).parent,
            pref_len,
            self.node(node_idx).pos,
        );
        self.set_child(insert_node, node_idx, word.as_bytes()[split_word_pos]);

        self.set_child(
            self.node(node_idx).parent,
            insert_node,
            word.as_bytes()[self.node(node_idx).pos],
        );

        self.update_edge(node_idx, insert_node, suf_len, split_word_pos);
        insert_node
    }
    #[inline]
    fn skip_walk(
        &self,
        word: &str,
        link: &mut NodeIdx,
        word_pos: &mut usize,
        edge_len: &mut usize,
    ) {
        loop {
            let key = word.as_bytes()[*word_pos];
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
    fn try_transfer_to<'r, 's, 't>(
        &'r mut self,
        word: &'t str,
        s: &'s mut State,
        ch: u8,
        is_online: bool,
    ) -> Transfer<'r, 's, 't> {
        if (is_online && self.node(s.node_idx).len() != usize::MAX && self.is_end_edge(&s))
            || (!is_online && self.is_end_edge(&s))
        {
            if self.node(s.node_idx).children.contains_key(&ch) {
                s.node_idx = self.to_node(s.node_idx, ch);
                s.edge_pos = self.node(s.node_idx).pos + 1;
                return Transfer::Success;
            } else {
                return Transfer::StopInNode(self, word, s, ch, is_online);
            }
        }
        if word.as_bytes()[s.edge_pos] == ch {
            s.edge_pos += 1;
            Transfer::Success
        } else {
            Transfer::StopInEdge(self, word, s, ch, is_online)
        }
    }

    #[inline]
    fn find(&self, word: &str, find: &str, is_online: bool) -> Option<usize> {
        let (word, mut find) = (word.as_bytes(), find.as_bytes());
        let find_len = find.len();
        let mut node_idx = NodeIdx::root();
        loop {
            let node = self.node(node_idx);
            let mut edge_pos = node.pos;
            let end_edge_pos = if is_online && node.len == usize::MAX {
                word.len()
            } else {
                node.pos + node.len
            };
            while edge_pos < end_edge_pos && !find.is_empty() && word[edge_pos] == find[0] {
                edge_pos += 1;
                find = &find[1..];
            }

            if find.is_empty() {
                return Some(edge_pos - find_len);
            }
            if edge_pos != end_edge_pos {
                return None;
            }
            node_idx = self.try_to_node(node_idx, find[0])?;
        }
    }
}

enum Transfer<'r, 's, 't> {
    StopInEdge(&'r mut AloneSuffixTree, &'t str, &'s mut State, u8, bool),
    StopInNode(&'r mut AloneSuffixTree, &'t str, &'s mut State, u8, bool),
    Success,
}

impl<'r, 's, 't> Transfer<'r, 's, 't> {
    #[inline]
    fn if_transfer_not_success<F>(self, f: F) -> bool
    where
        F: Fn(Transfer) -> bool,
    {
        match self {
            Transfer::Success => false,
            Transfer::StopInEdge(..) | Transfer::StopInNode(..) => f(self),
        }
    }
    #[inline]
    fn create_transfer(self, ch_pos: usize) -> Self {
        match self {
            Transfer::StopInEdge(tree, word, s, ch, is_online) => {
                let suf_len = if is_online && tree.node(s.node_idx).len == usize::MAX {
                    usize::MAX
                } else {
                    tree.node(s.node_idx).pos + tree.node(s.node_idx).len - s.edge_pos
                };
                let pref_len = if is_online {
                    s.edge_pos - tree.node(s.node_idx).pos
                } else {
                    tree.node(s.node_idx).len - suf_len
                };
                s.node_idx = tree.split(word, s.node_idx, s.edge_pos, pref_len, suf_len);

                let current_suf_len = if is_online {
                    usize::MAX
                } else {
                    word.len() - ch_pos
                };
                let add_node = tree.add_node(s.node_idx, current_suf_len, ch_pos);
                tree.set_child(s.node_idx, add_node, ch);
                Transfer::StopInNode(tree, word, s, ch, is_online)
            }
            Transfer::StopInNode(tree, word, s, ch, is_online) => {
                let current_suf_len = if is_online {
                    usize::MAX
                } else {
                    word.len() - ch_pos
                };
                let add_node = tree.add_node(s.node_idx, current_suf_len, ch_pos);
                tree.set_child(s.node_idx, add_node, ch);
                Transfer::StopInNode(tree, word, s, ch, is_online)
            }
            Transfer::Success => unreachable!(),
        }
    }
    #[inline]
    fn to_link(self) -> bool {
        match self {
            Transfer::StopInNode(tree, word, s, _, is_online) => {
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
                            let suf_len = if is_online && tree.node(link).len == usize::MAX {
                                usize::MAX
                            } else {
                                tree.node(link).len - edge_len
                            };
                            let insert_node = tree.split(word, link, word_pos, edge_len, suf_len);
                            tree.update_link(s, insert_node);
                        }
                    }
                };
                true
            }
            Transfer::StopInEdge(..) | Transfer::Success => unreachable!(),
        }
    }
}

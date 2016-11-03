use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{self, Debug, Formatter};
use std::iter::FromIterator;
use std::sync::Arc;

#[cfg(test)]
extern crate rand;
#[cfg(test)]
#[macro_use]
extern crate quickcheck;

#[cfg(test)]
mod tests;

#[derive(Clone)]
pub struct Tree<K, V> {
    root: Link<K, V>,
}

#[derive(Clone, Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    left: Link<K, V>,
    right: Link<K, V>,
    color: Color,
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Color {
    Red,
    Black,
    DoubleBlack,
}

#[derive(Clone, Debug)]
enum Link<K, V> {
    Empty,
    Node(Arc<Node<K, V>>),
}

impl<K, V> Tree<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    pub fn new() -> Tree<K, V> {
        Tree {
            root: Link::Empty,
        }
    }

    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V> where K: Borrow<Q>, Q: Ord {
        self.get_by(|node_key| key.cmp(node_key.borrow())).map(|(_, v)| v)
    }

    pub fn get_by<F>(&self, compare: F) -> Option<(&K, &V)>
                     where F: for<'a> FnMut(&'a K) -> Ordering {
        self.root.get_by(compare)
    }

    pub fn insert(&self, key: K, value: V) -> Tree<K, V> {
        Tree {
            root: self.root.insert(key, value),
        }
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, K, V> {
        Iter {
            start: self.root.to_option(),
            stack: vec![],
        }
    }
}

impl<K, V> Link<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    fn get_by<F>(&self, mut compare: F) -> Option<(&K, &V)>
                 where F: for<'a> FnMut(&'a K) -> Ordering {
        match *self {
            Link::Empty => None,
            Link::Node(ref node) => {
                match compare(&node.key) {
                    Ordering::Less => node.left.get_by(compare),
                    Ordering::Greater => node.right.get_by(compare),
                    Ordering::Equal => Some((&node.key, &node.value)),
                }
            }
        }
    }

    fn insert(&self, key: K, value: V) -> Link<K, V> {
        match *self {
            Link::Empty => {
                Link::Node(Arc::new(Node {
                    key: key,
                    value: value,
                    left: Link::Empty,
                    right: Link::Empty,
                    color: Color::Red,
                }))
            }
            Link::Node(ref node) => {
                match key.cmp(&node.key) {
                    Ordering::Less => {
                        node.color.balance(node.key.clone(),
                                           node.value.clone(),
                                           node.left.insert(key, value),
                                           node.right.clone())
                    }
                    Ordering::Greater => {
                        node.color.balance(node.key.clone(),
                                           node.value.clone(),
                                           node.left.clone(),
                                           node.right.insert(key, value))
                    }
                    Ordering::Equal => {
                        Link::Node(Arc::new(Node {
                            key: key,
                            value: value,
                            left: node.left.clone(),
                            right: node.right.clone(),
                            color: node.color,
                        }))
                    }
                }
            }
        }
    }

    fn is_empty_or_double_black(&self) -> bool {
        match *self {
            Link::Empty => true,
            Link::Node(ref node) => node.color.is_double_black(),
        }
    }

    fn to_option(&self) -> Option<&Arc<Node<K, V>>> {
        match *self {
            Link::Empty => None,
            Link::Node(ref node) => Some(node),
        }
    }

    fn get_if_red(&self) -> Option<&Arc<Node<K, V>>> {
        match *self {
            Link::Node(ref node) if node.color.is_red() => Some(node),
            Link::Node(_) | Link::Empty => None,
        }
    }

    fn get_if_black(&self) -> Option<&Arc<Node<K, V>>> {
        match *self {
            Link::Node(ref node) if node.color.is_black() => Some(node),
            Link::Node(_) | Link::Empty => None,
        }
    }

    fn double_black_to_black(&self) -> Link<Arc<Node<K, V>>> {
        match *self {
            Link::Node(ref node) => {
                debug_assert!(node.color.is_double_black());
                Link::Node(Arc::new(Node {
                    key: self.key.clone(),
                    value: self.value.clone(),
                    left: self.left.clone(),
                    right: self.right.clone(),
                    color: Color::Black,
                }))
            }
            Link::Empty => Link::Empty,
        }
    }

    fn rearrangement(color: Color,
                     parent_key: K,
                     parent_value: V,
                     left_key: K,
                     left_value: V,
                     left_left: Link<K, V>,
                     left_right: Link<K, V>,
                     right_key: K,
                     right_value: V,
                     right_left: Link<K, V>,
                     right_right: Link<K, V>)
                     -> Link<K, V> {
        Link::Node(Arc::new(Node {
            key: parent_key,
            value: parent_value,
            left: Link::Node(Arc::new(Node {
                key: left_key,
                value: left_value,
                left: left_left,
                right: left_right,
                color: Color::Black,
            })),
            right: Link::Node(Arc::new(Node {
                key: right_key,
                value: right_value,
                left: right_left,
                right: right_right,
                color: Color::Black,
            })),
            color: color,
        }))
    }
}

impl Color {
    fn balance<K, V>(self, key: K, value: V, left: Link<K, V>, right: Link<K, V>) -> Link<K, V>
                     where K: Clone + PartialOrd + Ord, V: Clone {
        if self.is_black() {
            if let Some(ref left) = left.get_if_red() {
                if let Some(ref left_left) = left.left.get_if_red() {
                    return Link::rearrangement(Color::Red,
                                               left.key.clone(),
                                               left.value.clone(),
                                               left_left.key.clone(),
                                               left_left.value.clone(),
                                               left_left.left.clone(),
                                               left_left.right.clone(),
                                               key,
                                               value,
                                               left.right.clone(),
                                               right)
                }
                if let Some(ref left_right) = left.right.get_if_red() {
                    return Link::rearrangement(Color::Red,
                                               left_right.key.clone(),
                                               left_right.value.clone(),
                                               left.key.clone(),
                                               left.value.clone(),
                                               left.left.clone(),
                                               left_right.left.clone(),
                                               key,
                                               value,
                                               left_right.right.clone(),
                                               right)
                }
            }
            if let Some(ref right) = right.get_if_red() {
                if let Some(ref right_left) = right.left.get_if_red() {
                    return Link::rearrangement(Color::Red,
                                               right_left.key.clone(),
                                               right_left.value.clone(),
                                               key,
                                               value,
                                               left,
                                               right_left.left.clone(),
                                               right.key.clone(),
                                               right.value.clone(),
                                               right_left.right.clone(),
                                               right.right.clone())
                }
                if let Some(ref right_right) = right.right.get_if_red() {
                    return Link::rearrangement(Color::Red,
                                               right.key.clone(),
                                               right.value.clone(),
                                               key,
                                               value,
                                               left,
                                               right.left.clone(),
                                               right_right.key.clone(),
                                               right_right.value.clone(),
                                               right_right.left.clone(),
                                               right_right.right.clone())
                }
            }
        }

        if self.is_double_black() {
            if let Some(ref left) = left.get_if_red() {
                if let Some(ref left_right) = left.right.get_if_red() {
                    return Link::rearrangement(Color::Black,
                                               left_right.key.clone(),
                                               left_right.value.clone(),
                                               left.key.clone(),
                                               left.value.clone(),
                                               left.left.clone(),
                                               left_right.left.clone(),
                                               key,
                                               value,
                                               left_right.right.clone(),
                                               right)
                }
            }
            if let Some(ref right) = right.get_if_red() {
                if let Some(ref right_left) = right.left.get_if_red() {
                    return Link::rearrangement(Color::Black,
                                               right_left.key.clone(),
                                               right_left.value.clone(),
                                               key,
                                               value,
                                               left,
                                               right_left.left.clone(),
                                               right.key.clone(),
                                               right.value.clone(),
                                               right_left.right.clone(),
                                               right.right.clone())
                }
            }
        }

        Link::Node(Arc::new(Node {
            key: key,
            value: value,
            left: left,
            right: right,
            color: self,
        }))
    }

    fn rotate<K, V>(self, key: K, value: V, left: Link<K, V>, right: Link<K, V>) -> Link<K, V>
                    where K: Clone + PartialOrd + Ord, V: Clone {
        if self.is_red() {
            if left.is_empty_or_double_black() {
                if let Some(right) = right.get_if_black() {
                    return Color::Black.balance(right.key.clone(),
                                                right.value.clone(),
                                                Link::Node(Arc::new(Node {
                                                    color: Color::Red,
                                                    key: key,
                                                    value: value,
                                                    left: left.double_black_to_black(),
                                                    right: right.left.clone(),
                                                })),
                                                right.right.clone())
                }
            }
            if right.is_empty_or_double_black() {
                if let Some(left) = left.get_if_black() {
                    return Color::Black.balance(left.key.clone(),
                                                left.value.clone(),
                                                left.left.clone(),
                                                Link::Node(Arc::new(Node {
                                                    color: Color::Red,
                                                    key: key,
                                                    value: value,
                                                    left: left.right.clone(),
                                                    right: right.double_black_to_black(),
                                                })))
                }
            }
        } else if self.is_black() {
            if left.is_empty_or_double_black() {
                // Second case, Figure 7
                if let Some(right) = right.get_if_black() {
                    return Color::DoubleBlack.balance(right.key.clone(),
                                                      right.value.clone(),
                                                      Link::Node(Arc::new(Node {
                                                          color: Color::Red,
                                                          key: key,
                                                          value: value,
                                                          left: left.double_black_to_black(),
                                                          right: right.left.clone(),
                                                      })),
                                                      right.right.clone())
                }
                // Third case, Figure 9
                if let Some(right) = right.get_if_red() {
                    if let Some(right_left) = right.left.get_if_black() {
                        return Link::Node(Arc::new(Node {
                            color: Color::Black,
                            key: right.key.clone(),
                            value: right.value.clone(),
                            left: Color::Black.balance(right_left.key.clone(),
                                                       right_left.value.clone(),
                                                       Link::Node(Arc::new(Node {
                                                           color: Color::Red,
                                                           key: key,
                                                           value: value,
                                                           left: left.double_black_to_black(),
                                                           right: right_left.left.clone(),
                                                       })),
                                                       right_left.right.clone()),
                            right: right.right.clone(),
                        }))
                    }
                }
            }
            if right.is_empty_or_double_black() {
                // Second case, Figure 7
                if let Some(left) = left.get_if_black() {
                    return Color::DoubleBlack.balance(left.key.clone(),
                                                      left.value.clone(),
                                                      left.left.clone(),
                                                      Link::Node(Arc::new(Node {
                                                          color: Color::Red,
                                                          key: key,
                                                          value: value,
                                                          left: left.right.clone(),
                                                          right: right.double_black_to_black(),
                                                      })))
                }
                // Third case, Figure 9
                if let Some(left) = left.get_if_red() {
                    if let Some(left_right) = left.right.get_if_black() {
                        return Link::new(Arc::new(Node {
                            color: Color::Black,
                            key: left.key.clone(),
                            value: left.value.clone(),
                            left: left.left.clone(),
                            right: Color::Black.balance(left_right.key.clone(),
                                                        left_right.value.clone(),
                                                        left_right.left.clone(),
                                                        Link::Node(Arc::new(Node {
                                                            color: Color::Red,
                                                            key: key,
                                                           value: value,
                                                            left: left_right.right.clone(),
                                                            right: right.double_black_to_black(),
                                                        }))),
                        }))
                    }
                }
            }
        }

        Link::Node(Arc::new(Node {
            key: key,
            value: value,
            left: left,
            right: right,
            color: self,
        }))
    }

    fn is_red(self) -> bool {
        self == Color::Red
    }

    fn is_black(self) -> bool {
        self == Color::Black
    }

    fn is_double_black(self) -> bool {
        self == Color::DoubleBlack
    }
}

pub struct Iter<'a, K, V> where K: 'a, V: 'a {
    start: Option<&'a Arc<Node<K, V>>>,
    stack: Vec<&'a Arc<Node<K, V>>>,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<(&'a K, &'a V)> {
        let mut current = self.start.take();
        while !self.stack.is_empty() || current.is_some() {
            match current {
                Some(node) => {
                    self.stack.push(node);
                    current = node.left.to_option()
                }
                None => {
                    let node = self.stack.pop().unwrap();
                    let item = (&node.key, &node.value);
                    self.start = node.right.to_option();
                    return Some(item)
                }
            }
        }
        None
    }
}

impl<K, V> FromIterator<(K, V)> for Tree<K, V> where K: Clone + PartialOrd + Ord, V: Clone {
    fn from_iter<T>(iter: T) -> Tree<K, V> where T: IntoIterator<Item = (K, V)> {
        let mut tree = Tree::new();
        for (key, value) in iter {
            tree = tree.insert(key, value)
        }
        tree
    }
}

impl<K, V> Debug for Tree<K, V> where K: Clone + PartialOrd + Ord + Debug, V: Clone + Debug {
    fn fmt(&self, formatter: &mut Formatter) -> Result<(), fmt::Error> {
        try!(write!(formatter, "["));
        let mut iter = self.iter();
        if let Some(pair) = iter.next() {
            try!(pair.fmt(formatter));
            for pair in iter {
                try!(write!(formatter, ", {:?}", pair))
            }
        }
        write!(formatter, "]")
    }
}


use std::fmt::Debug;

#[derive(Clone)]
pub enum Tree<T>
where
    T: BeTree + Debug + Clone,
{
    Node(Vec<Self>),
    Dot(Box<T>),
}
impl<T> Tree<T>
where
    T: BeTree + Debug + Clone,
{
    pub fn push(&mut self, e: T) {
        let new = Self::Dot(Box::new(e));
        match self {
            Tree::Node(_vec) => _vec.push(new),
            Tree::Dot(_) => *self = Self::Node(vec![self.clone(), new]),
        }
    }
    pub fn open_to_vec(&self) -> Vec<T> {
        match self {
            Tree::Node(_vec) => {
                let mut ret: Vec<T> = Vec::new();
                for e in _vec {
                    ret.append(&mut e.open_to_vec());
                }
                return ret;
            }
            Tree::Dot(_e) => vec![*_e.to_owned()],
        }
    }
    pub fn node_to_vec(&self) -> Vec<Tree<T>> {
        match self {
            Tree::Node(_e) => _e.clone(),
            Tree::Dot(_e) => vec![self.clone()],
        }
    }
}
impl<T> Debug for Tree<T>
where
    T: BeTree + Debug + Clone,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Node(arg0) => {
                let mut str = String::from("Node [");
                for tree in arg0 {
                    str += " ";
                    str += &format!("{:?}", tree);
                    str += " ";
                }
                str += "]";
                write!(f, "{}", str)
            }
            Self::Dot(arg0) => {
                write!(f, "{:?}", arg0)
            }
        }
    }
}
pub trait BeTree
where
    Self: Sized + Debug + Clone,
{
    // type Self = dyn Tree;
    fn deep(&self) -> usize;
    fn is_left_part(&self) -> bool;
    fn is_right_part(&self) -> bool;
    fn build_deep_tree(ss: Vec<Self>, deep: usize) -> Tree<Self> {
        let mut deep_now;
        let mut wait = 0;
        let mut ret_vec: Vec<Tree<Self>> = Vec::new();
        for i in 0..ss.len() {
            deep_now = ss[i].deep();
            if deep_now > deep {
                if wait == 0 {
                    ret_vec.push(Self::build_deep_tree(ss[i..].to_vec(), deep_now))
                }
                wait += 1;
            } else if deep_now < deep {
                if wait == 0 {
                    return Tree::Node(ret_vec);
                }
                wait -= 1;
            } else if wait == 0 {
                ret_vec.push(Tree::Dot(Box::new(ss[i].clone())))
            }
        }
        return Tree::Node(ret_vec);
    }
    fn build_node_tree(ss: Vec<Self>, deep: usize) -> Tree<Self> {
        let mut deep_now = deep;
        let mut wait = false;
        let mut ret = Vec::new();
        for s in &ss {
            if s.is_left_part() {
                deep_now += 1;
                continue;
            } else if s.is_right_part() {
                deep_now -= 1;
                continue;
            }
            if deep < deep_now && !wait {
                ret.push(Self::build_node_tree(ss.clone(), deep_now));
                wait = true;
            } else {
                wait = false;
                if deep == deep_now {
                    ret.push(Tree::Dot(Box::new(s.clone())));
                } else if deep > deep_now {
                    return Tree::Node(ret);
                }
            }
        }

        return Tree::Node(ret);
    }
}

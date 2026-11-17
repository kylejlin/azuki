pub type S<'src> = &'src str;
pub type Node<'src> = Vec<S<'src>>;

pub fn reduce_r1<'src>(left: S<'src>, right: S<'src>) -> Node<'src> {
    vec![left, right]
}

pub fn pop_and_reduce<'src>(mut nodes: Node<'src>, terminal: S<'src>) -> Node<'src> {
    nodes.push(terminal);
    nodes
}

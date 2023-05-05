#[derive(Clone)]
pub struct BroadcastMsg<T> {
    pub target: Target,
    pub content: T,
}

#[derive(Clone)]
pub enum Target {
    All,
    One(i64),
}

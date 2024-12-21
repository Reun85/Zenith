#[derive(Debug, derive_more::Deref, derive_more::DerefMut, derive_more::Into, Clone, Copy)]
pub struct Promise<Result, Description = (), Intermediary = ()> {
    #[deref]
    #[into]
    #[deref_mut]
    pub result: Option<Result>,
    pub description: Description,
    pub intermediary: Intermediary,
}

impl<R, D, I> Promise<R, D, I>
where
    I: std::default::Default,
{
    pub fn new(desc: D) -> Self {
        Self {
            result: None,
            description: desc,
            intermediary: I::default(),
        }
    }
    pub fn unwrap(self) -> R {
        self.result.unwrap()
    }
    pub fn result(self) -> Option<R> {
        self.result
    }
}

#[derive(Debug, derive_more::Deref, derive_more::DerefMut, derive_more::Into, Clone)]
pub struct Promise<Result, Description = (), Intermediary = ()> {
    #[deref]
    #[into]
    #[deref_mut]
    pub result: std::cell::RefCell<Option<Result>>,
    pub description: Description,
    pub intermediary: std::cell::RefCell<Intermediary>,
}

impl<R, D, I> Promise<R, D, I>
where
    I: std::default::Default,
{
    pub fn new(desc: D) -> Self {
        Self {
            result: None.into(),
            description: desc,
            intermediary: I::default().into(),
        }
    }
    pub fn new_rc(desc: D) -> std::rc::Rc<Self> {
        std::rc::Rc::new(Self {
            result: None.into(),
            description: desc,
            intermediary: I::default().into(),
        })
    }
    pub fn new_rfrc(desc: D) -> std::rc::Rc<std::cell::RefCell<Self>> {
        std::rc::Rc::new(std::cell::RefCell::new(Self {
            result: None.into(),
            description: desc,
            intermediary: I::default().into(),
        }))
    }
    pub fn unwrap(self) -> R {
        self.result.into_inner().unwrap()
    }
    pub fn result(self) -> Option<R> {
        self.result.into_inner()
    }
}

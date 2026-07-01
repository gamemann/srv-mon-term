#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TuiInterfaceContext<T> {
    pub interface: T,
}

impl<T> TuiInterfaceContext<T> {
    pub fn new() -> Self
    where
        T: Default,
    {
        Self {
            interface: T::default(),
        }
    }

    pub fn new_with_opts<V>(opts: V) -> Self
    where
        T: From<V>,
    {
        Self {
            interface: T::from(opts),
        }
    }
}

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
}

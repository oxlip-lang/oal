pub trait TryEach {
    type Item;

    fn try_each<F, T, E>(self, f: F) -> Result<(), E>
    where
        F: FnMut(Self::Item) -> Result<T, E>;
}

impl<C, U> TryEach for C
where
    C: IntoIterator<Item = U>,
{
    type Item = U;

    fn try_each<F, T, E>(self, mut f: F) -> Result<(), E>
    where
        F: FnMut(Self::Item) -> Result<T, E>,
    {
        self.into_iter()
            .map(|e| f(e))
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    }
}

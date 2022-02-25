pub trait TryEach<'a, U: 'a> {
    fn try_each<F, T, E>(self, f: F) -> Result<(), E>
    where
        F: FnMut(U) -> Result<T, E>;
}

impl<'a, C, U> TryEach<'a, &'a mut U> for &'a mut C
where
    &'a mut C: IntoIterator<Item = &'a mut U>,
{
    fn try_each<F, T, E>(self, mut f: F) -> Result<(), E>
    where
        F: FnMut(&'a mut U) -> Result<T, E>,
    {
        self.into_iter()
            .map(|e| f(e))
            .collect::<Result<Vec<_>, _>>()
            .map(|_| ())
    }
}

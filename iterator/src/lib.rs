pub trait IteratorExt: Iterator + Sized {
    // Sized is need
    fn my_flatten(self) -> Flatten<Self>
    where
        Self: Iterator<Item: IntoIterator>;
}

impl<T> IteratorExt for T
where
    T: Iterator,
{
    fn my_flatten(self) -> Flatten<Self>
    where
        Self: Iterator<Item: IntoIterator>,
    {
        Flatten::new(self)
    }
}

pub struct Flatten<I: Iterator<Item: IntoIterator>> {
    inner: FlattenCompat<I, <I::Item as IntoIterator>::IntoIter>,
}

impl<I: Iterator<Item: IntoIterator>> Flatten<I> {
    fn new(iter: I) -> Self {
        Self {
            inner: FlattenCompat::new(iter),
        }
    }
}

struct FlattenCompat<I, U> {
    outer_iter: I,
    front_iter: Option<U>,
    back_iter: Option<U>,
}

impl<I, U> FlattenCompat<I, U>
where
    I: Iterator,
{
    fn new(iter: I) -> Self {
        Self {
            outer_iter: iter,
            front_iter: None,
            back_iter: None,
        }
    }
}

impl<I, U> Iterator for Flatten<I>
where
    I: Iterator<Item: IntoIterator<Item = U::Item, IntoIter = U>>,
    U: Iterator,
{
    type Item = U::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<I, U> DoubleEndedIterator for Flatten<I>
where
    I: DoubleEndedIterator<Item: IntoIterator<Item = U::Item, IntoIter = U>>,
    U: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<I, U> Iterator for FlattenCompat<I, U>
where
    I: Iterator<Item: IntoIterator<Item = U::Item, IntoIter = U>>,
    U: Iterator,
{
    type Item = U::Item;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut front_iter) = self.front_iter {
                let next = front_iter.next();
                if next.is_some() {
                    return next;
                }
                self.front_iter = None;
            }

            if let Some(next_front_iter) = self.outer_iter.next() {
                self.front_iter = Some(next_front_iter.into_iter());
            } else {
                return self.back_iter.as_mut()?.next();
            }
        }
    }
}

impl<I, U> DoubleEndedIterator for FlattenCompat<I, U>
where
    I: DoubleEndedIterator<Item: IntoIterator<Item = U::Item, IntoIter = U>>,
    U: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(ref mut back_iter) = self.back_iter {
                let next = back_iter.next_back();
                if next.is_some() {
                    return next;
                }
                self.back_iter = None;
            }

            if let Some(next_back_iter) = self.outer_iter.next_back() {
                self.back_iter = Some(next_back_iter.into_iter());
            } else {
                return self.front_iter.as_mut()?.next_back();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let data = vec![vec![1, 2, 3, 4], vec![5, 6]];
        let same_data = vec![vec![1, 2, 3, 4], vec![5, 6]];
        let expect: Vec<u8> = data.into_iter().flatten().collect();
        let res: Vec<u8> = same_data.into_iter().my_flatten().collect();
        assert_eq!(expect, res);
    }

    #[test]
    fn empty() {
        assert_eq!(std::iter::empty::<Vec<()>>().my_flatten().count(), 0);
    }

    #[test]
    fn reverse() {
        assert_eq!(
            std::iter::once(vec!["a", "b"])
                .my_flatten()
                .rev()
                .collect::<Vec<_>>(),
            vec!["b", "a"]
        );
    }

    #[test]
    fn both_ends() {
        let mut iter0 = vec![vec!["a1", "a2", "a3"], vec!["b1", "b2", "b3"]]
            .into_iter()
            .my_flatten();
        let mut iter1 = vec![vec!["a1", "a2", "a3"], vec!["b1", "b2", "b3"]]
            .into_iter()
            .flatten();
        assert_eq!(iter0.next(), iter1.next());
        assert_eq!(iter0.next_back(), iter1.next_back());
        assert_eq!(iter0.next(), iter1.next());
        assert_eq!(iter0.next_back(), iter1.next_back());
        assert_eq!(iter0.next(), iter1.next());
        assert_eq!(iter0.next_back(), iter1.next_back());
        assert_eq!(iter0.next(), iter1.next());
        assert_eq!(iter0.next_back(), iter1.next_back());
    }
}

use std::ops::Index;

pub struct Array<A: Index<usize>> {
    size: usize,
    backing: A,
}

impl<A: Index<usize>> Array<A> {
    fn new(size: usize, backing: A) -> Self {
        Self { size, backing }
    }

    fn coords(size: usize) -> impl Iterator<Item = (usize, usize, usize)> {
        (0..size)
            .flat_map(move |x| (0..size).map(move |y| (x, y)))
            .flat_map(move |(x, y)| (0..size).map(move |z| (x, y, z)))
    }
}

impl<T> Array<Vec<T>> {
    pub fn create_from<F: FnMut((usize, usize, usize)) -> T>(size: usize, mut func: F) -> Self {
        Self::new(size, Self::coords(size).map(|x| func(x)).collect::<Vec<_>>())
    }
}

impl<A: Index<usize>> Index<(usize, usize, usize)> for Array<A> {
    type Output = A::Output;

    fn index(&self, index: (usize, usize, usize)) -> &Self::Output {
        if index.0 >= self.size || index.1 >= self.size || index.2 >= self.size {
            panic!(
                "Index out of range (size {}): {}, {}, {}",
                self.size, index.0, index.1, index.2
            );
        }
        &self.backing[self.size * self.size * index.0 + self.size * index.1 + index.2]
    }
}

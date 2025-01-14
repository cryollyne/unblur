use std::ops::{Add, Div, Mul, Neg, Sub};

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Vector<T, const N: usize> {
    pub data: [T; N],
}

impl<T, const N: usize> Vector<T, N> {
    pub fn new(data: [T; N]) -> Vector<T, N> {
        Vector::<T, N> { data }
    }

    pub fn map_with(mut self, other: Self, f: fn(T,T)->T) -> Self
    where
        T: Clone
    {
        self.data.iter_mut()
            .zip(other.data)
            .for_each(|(x,y)| {
                *x = f(x.clone(), y)
            });
        self
    }

    pub fn map(mut self, f: fn(T) -> T) -> Self
    where
        T: Clone
    {
        self.data.iter_mut()
            .for_each(|x| {
                *x = f(x.clone())
            });
        self
    }
}

impl<T, const N: usize> Default for Vector<T, N>
where
    T: Default + Clone + Copy,
{
    fn default() -> Self {
        Vector::<T, N> {
            data: [T::default(); N],
        }
    }
}

impl<T, const N: usize> Add for Vector<T, N>
where
    T: Add<Output = T>,
{
    type Output = Vector<T, N>;

    fn add(self, rhs: Self) -> Self::Output {
        let added = self
            .data
            .into_iter()
            .zip(rhs.data)
            .map(|(x, y)| x + y)
            .collect::<Vec<_>>()
            .try_into();
        let data = match added {
            Ok(d) => d,
            _ => panic!("unknown error"),
        };
        Vector::<T, N> { data }
    }
}

impl<T, const N: usize> Sub for Vector<T, N>
where
    T: Sub<Output = T>,
{
    type Output = Vector<T, N>;

    fn sub(self, rhs: Self) -> Self::Output {
        let added = self
            .data
            .into_iter()
            .zip(rhs.data)
            .map(|(x, y)| x - y)
            .collect::<Vec<_>>()
            .try_into();
        let data = match added {
            Ok(d) => d,
            _ => panic!("unknown error"),
        };
        Vector::<T, N> { data }
    }
}

impl<T, const N: usize> Mul for Vector<T, N>
where
    T: Mul<Output = T>,
{
    type Output = Vector<T, N>;

    fn mul(self, rhs: Self) -> Self::Output {
        let added = self
            .data
            .into_iter()
            .zip(rhs.data)
            .map(|(x, y)| x * y)
            .collect::<Vec<_>>()
            .try_into();
        let data = match added {
            Ok(d) => d,
            _ => panic!("unknown error"),
        };
        Vector::<T, N> { data }
    }
}

impl<T, const N: usize> Div for Vector<T, N>
where
    T: Div<Output = T>,
{
    type Output = Vector<T, N>;

    fn div(self, rhs: Self) -> Self::Output {
        let added = self
            .data
            .into_iter()
            .zip(rhs.data)
            .map(|(x, y)| x / y)
            .collect::<Vec<_>>()
            .try_into();
        let data = match added {
            Ok(d) => d,
            _ => panic!("unknown error"),
        };
        Vector::<T, N> { data }
    }
}

impl<T, const N: usize> Neg for Vector<T, N>
where
    T: Neg<Output = T>,
{
    type Output = Vector<T, N>;

    fn neg(self) -> Self::Output {
        let added = self
            .data
            .into_iter()
            .map(|x| -x)
            .collect::<Vec<_>>()
            .try_into();
        let data = match added {
            Ok(d) => d,
            _ => panic!("unknown error"),
        };
        Vector::<T, N> { data }
    }
}

use std::u128;

pub type Bits = Box<[u128]>;

pub trait BoxSliceExt {
    fn from_bit_siter(t: impl ExactSizeIterator<Item = bool>) -> Self;
    fn zeros(len: usize) -> Self;
    fn ones(len: usize) -> Self;
    fn count_ones(&self) -> u32;
    fn conjunction_assign(&mut self, other: &Self);
    fn union_assign(&mut self, other: &Self);
    fn difference_assign(&mut self, other: &Self);
    fn subset(&self, other: &Self) -> bool;
    fn get(&self, index: usize) -> bool;
    
}
fn ceildiv(a: usize, b: usize) -> usize {
    (a + b - 1) / b
}

impl BoxSliceExt for Box<[u128]> {
    fn zeros(len: usize) -> Self {
        (0..ceildiv(len, u128::BITS as usize)).map(|_| 0u128).collect()
    }
    fn ones(len: usize) -> Self {
        let mut result: Self = (0..ceildiv(len, u128::BITS as usize)).map(|_| u128::MAX).collect();
        if len % u128::BITS as usize != 0 {
            result.last_mut().map(|x| *x &= (1 << (len % u128::BITS as usize)) - 1);
        }
        result
    }

    fn count_ones(&self) -> u32 {
        self.iter().map(|x| x.count_ones()).sum()
    }
    
    fn conjunction_assign(&mut self, other: &Self) {
        self.iter_mut().zip(other.iter()).for_each(|(i, j)| *i &= j);
    }
    fn union_assign(&mut self, other: &Self) {
        self.iter_mut().zip(other.iter()).for_each(|(i, j)| *i |= j);
    }
    
    fn difference_assign(&mut self, other: &Self) {
        self.iter_mut().zip(other.iter()).for_each(|(i, j)| *i &= !j);
    }

    fn subset(&self, other: &Self) -> bool {
        other.iter().zip(self.iter()).all(|(i, j)| i & j == *j)
    }
    
    fn from_bit_siter(t: impl ExactSizeIterator<Item = bool>) -> Self {
        let mut res = vec![0u128; ceildiv(t.len(), 128)];
        for (i, b) in t.enumerate() {
            if b {
                res[i / 128] |= 1 << (i % 128);
            }
        }
        res.into_boxed_slice()
    }
    
    fn get(&self, index: usize) -> bool {
        self[index / 128] & (1 << (index % 128)) != 0
    }
}

pub fn boxed_ones(size: usize) -> Box<[u128]> {
    let l = ceildiv(size, u128::BITS as usize);
    let rem = size as u32 % u128::BITS;
    (0..l).map(|i| if i + 1 == l { (1 << rem) - 1 } else { u128::MAX }).collect()
}

#[cfg(test)]
mod test {
    use super::BoxSliceExt;

    #[test]
    fn test() {
        for i in 0..=256 {
            let a = Box::ones(i);
            assert_eq!(i, a.iter().map(|x| x.count_ones() as usize).sum::<usize>());
        }
    }
}
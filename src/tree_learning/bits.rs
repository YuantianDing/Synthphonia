use std::u128;

pub type Bits = Box<[u128]>;

/// A trait defining extended operations on box slices, particularly focused on bit manipulation. 
/// 
/// 
/// This trait provides methods to create a new instance from an iterator of boolean values, initialize slices filled with zeros or ones, and count the number of ones in the slice. 
/// It also includes methods to perform bitwise conjunction, union, and difference operations in place, while offering functionality to check if a slice is a subset of another and to access the boolean value at a specific index. 
/// These operations facilitate efficient manipulation and querying of boxed slices as bit arrays.
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
/// Calculates the ceiling of the division of two unsigned integers. 
/// 
/// This function takes two parameters, `a` and `b`, both of the type `usize`, and returns the smallest integer greater than or equal to the result of dividing `a` by `b`. 
/// It utilizes the `div_ceil` method to perform the calculation, which inherently manages any need to round up the result of the division when `a` is not perfectly divisible by `b`. 
/// This function is useful in scenarios where precise ceiling division is required.
/// 
fn ceildiv(a: usize, b: usize) -> usize {
    a.div_ceil(b)
}

impl BoxSliceExt for Box<[u128]> {
    /// Creates a boxed slice of `u128` integers, initializing each value to zero. 
    /// 
    /// It achieves this by calculating the necessary length in terms of `u128` blocks to cover the specified `len` in bits and then mapping over this range, collecting zeros into a boxed slice. 
    /// The process involves using a ceiling division to ensure sufficient space is allocated, corresponding to the bit width of `u128`.
    /// 
    fn zeros(len: usize) -> Self {
        (0..ceildiv(len, u128::BITS as usize)).map(|_| 0u128).collect()
    }
    /// Creates a new boxed slice of `u128` integers each filled with the value `u128::MAX`, with partial bit masking applied to the last element if the specified length does not directly align with a multiple of `u128`'s bit size. 
    /// 
    /// This function initializes a vector by determining the number of `u128` elements required via ceiling division, fills it with maximum possible 128-bit unsigned integers, and then collects this into a boxed slice. 
    /// If the intended length in bits leaves a remainder that isn't a full `u128`, the final element is appropriately masked to ensure only the requisite bits are set to one, maintaining the defined length.
    /// 
    fn ones(len: usize) -> Self {
        let mut result: Self = (0..ceildiv(len, u128::BITS as usize)).map(|_| u128::MAX).collect();
        if len % u128::BITS as usize != 0 {
            result.last_mut().map(|x| *x &= (1 << (len % u128::BITS as usize)) - 1);
        }
        result
    }

    /// Provides a method that calculates the total number of one-bits in a boxed slice of `u128` integers. 
    /// 
    /// It iterates over each `u128` value in the slice, applies the `count_ones` method to obtain the number of one-bits for each element, and sums these counts to return the cumulative total as a `u32`. 
    /// This operation enables efficient bit counting across potentially large numeric sequences.
    /// 
    fn count_ones(&self) -> u32 {
        self.iter().map(|x| x.count_ones()).sum()
    }
    
    /// Performs an in-place bitwise conjunction between two slices of `u128`. 
    /// 
    /// This operation modifies the elements of the caller slice by iterating through it, paired with elements from the `other` slice, applying a bitwise AND operation, and storing the result back into the caller slice elements. 
    /// Each element in the slice is processed in sequence such that their corresponding positions between the two slices are combined using the AND operation.
    /// 
    fn conjunction_assign(&mut self, other: &Self) {
        self.iter_mut().zip(other.iter()).for_each(|(i, j)| *i &= j);
    }
    /// Performs an in-place union operation between two boxed slices. 
    /// 
    /// This method iterates over each element of the boxed slice and the corresponding element of another boxed slice, applying the bitwise OR operation. 
    /// The result of this operation updates the elements of the boxed slice on which the method is called (`self`). 
    /// This allows for the combination of elements in two slices, storing the union of each pair of elements from the slices into the first slice. 
    /// This operation assumes both slices have the same length, as it utilizes the `zip` method to combine elements from `self` and `other`.
    /// 
    fn union_assign(&mut self, other: &Self) {
        self.iter_mut().zip(other.iter()).for_each(|(i, j)| *i |= j);
    }
    
    /// Calculates the difference between the current boxed slice of `u128` integers and another slice, assigning the result to the current instance. 
    /// This method iterates through both slices in parallel, applying a bitwise AND with the negation operation on each corresponding pair of elements, thereby effectively subtracting the bit representation of the elements from `other` out of the current slice's elements.
    fn difference_assign(&mut self, other: &Self) {
        self.iter_mut().zip(other.iter()).for_each(|(i, j)| *i &= !j);
    }

    /// Determines if one boxed slice of `u128` integers is a subset of another. 
    /// 
    /// This functionality is executed by comparing each integer in the `other` slice with the corresponding integer in `self` slice. 
    /// The method utilizes bitwise AND operation to ascertain that every integer in `self` is contained within the same indexed position in `other`. 
    /// The method returns true only if this condition holds for every pair of integers throughout the slices, indicating that `self` is indeed a subset of `other`.
    /// 
    fn subset(&self, other: &Self) -> bool {
        other.iter().zip(self.iter()).all(|(i, j)| i & j == *j)
    }
    
    /// Constructs a boxed slice of `u128` integers from an iterator of boolean values. 
    /// 
    /// The function takes an `ExactSizeIterator` of `bool` items and initializes a `Vec<u128>` large enough to represent each boolean value as a bit within `u128` segments. 
    /// It calculates the necessary number of `u128` elements using a ceiling division based on the iterator's length divided by 128. 
    /// As it iterates over the boolean items, it appropriately sets bits in the `u128` segments depending on their indexes. 
    /// This results in a bit-packed representation of the iterator's boolean sequence, which is then converted into a boxed slice for returned storage efficiency.
    /// 
    fn from_bit_siter(t: impl ExactSizeIterator<Item = bool>) -> Self {
        let mut res = vec![0u128; ceildiv(t.len(), 128)];
        for (i, b) in t.enumerate() {
            if b {
                res[i / 128] |= 1 << (i % 128);
            }
        }
        res.into_boxed_slice()
    }
    
    /// Returns the boolean value of the bit at the given index for a boxed slice of `u128` integers. 
    /// 
    /// This function calculates the position of the bit within the slice by dividing the `index` by 128 to locate the correct `u128` integer and then using the modulus operation to determine the bit position within that integer. 
    /// It then uses bitwise operations to isolate the bit at the specified index and checks whether it is set, returning `true` if the bit is `1` and `false` otherwise.
    /// 
    fn get(&self, index: usize) -> bool {
        self[index / 128] & (1 << (index % 128)) != 0
    }
}

/// Creates a boxed slice of `u128` filled with binary ones, with the specified length in bits. 
/// 
/// The function calculates the number of `u128` elements needed to represent the given `size` in bits, taking into account the bit width of `u128`. 
/// It uses a ceiling division to ensure all bits are accommodated. 
/// Each element of the boxed slice is filled with ones, except for the last element, which is calculated to fit only the remaining bits required. 
/// The resulting boxed slice represents a sequence of `u128` values with all bits set to one up to the specified bit length.
/// 
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
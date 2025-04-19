use ext_trait::extension;


/// This structure wraps an iterator alongside a separator string to facilitate joining elements with a specified delimiter during formatting. 
/// 
/// 
/// It holds a cloneable iterator over items and a string slice that serves as the separator between the items when they are formatted. 
/// This design supports multiple passes of the iterator during formatting operations and enables implementations of formatting traits to output the joined elements either using their debug or display representations.
pub struct FmtJoin<'i, T: Iterator<Item=I> + Clone, I>(T, &'i str);

impl<'i, T: Iterator<Item=I> + Clone, I: std::fmt::Debug> std::fmt::Debug for FmtJoin<'i, T, I> {
    /// Formats a sequence of items by concatenating their Debug representations with a specified separator. 
    /// This function takes a formatter and writes the Debug form of the first element, then iterates over the remaining items, prefixing each with the provided separator before writing their Debug representations.
    /// 
    /// It clones the underlying iterator, ensuring the original sequence is preserved, and gracefully handles cases where the iterator may be empty by simply returning a successful formatting result.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone();
        if let Some(a) = iter.next() {
            write!(f, "{:?}", a)?;
            for p in iter {
                write!(f, "{}{:?}", self.1, p)?;
            }
        }
        Ok(())
    }
}

impl<'i, T: Iterator<Item=I> + Clone, I: std::fmt::Display> std::fmt::Display for FmtJoin<'i, T, I> {
    /// Formats and writes the joined representation of iterator elements with a given separator. 
    /// 
    /// 
    /// Writes the first element without a preceding separator and then appends each subsequent element prefixed by the specified separator by iterating over a cloned version of the stored iterator, thereby yielding a formatted string representation.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut iter = self.0.clone();
        if let Some(a) = iter.next() {
            write!(f, "{}", a)?;
            for p in iter {
                write!(f, "{}{}", self.1, p)?;
            }
        }
        Ok(())
    }
}


#[extension(pub trait FmtJoinIter)]
impl<T: Iterator + Clone> T {
    /// Creates a join formatter for an iterator. 
    /// This method consumes the iterator, associates it with a given separator string, and returns a wrapper that, when formatted, joins the elements of the iterator using the provided separator.
    fn fmtjoin<'i>(self, sep: &'i str) -> FmtJoin<'i, T, T::Item> {
        FmtJoin(self, sep)
    }
}
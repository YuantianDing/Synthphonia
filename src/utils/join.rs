use ext_trait::extension;


pub struct FmtJoin<'i, T: Iterator<Item=I> + Clone, I>(T, &'i str);

impl<'i, T: Iterator<Item=I> + Clone, I: std::fmt::Debug> std::fmt::Debug for FmtJoin<'i, T, I> {
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
    fn fmtjoin<'i>(self, sep: &'i str) -> FmtJoin<'i, T, T::Item> {
        FmtJoin(self, sep)
    }
}
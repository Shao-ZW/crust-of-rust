pub struct Split<'a, D> {
    remainder: Option<&'a str>,
    delimiter: D,
}

pub trait Delimiter {
    fn find_next(&self, s: &str) -> Option<(usize, usize)>;
}

pub fn split<D: Delimiter>(s: &str, delimiter: D) -> Split<D> {
    Split {
        remainder: Some(s),
        delimiter,
    }
}

impl<'a, D: Delimiter> Iterator for Split<'a, D> {
    type Item = &'a str;
    fn next(&mut self) -> Option<Self::Item> {
        // let s = &mut self.remainder?;    // wrong
        let s = self.remainder.as_mut()?;
        if let Some((start, end)) = self.delimiter.find_next(*s) {
            let ret = &s[..start];
            *s = &s[end..];
            Some(ret)
        } else {
            self.remainder.take()
        }
    }
}

impl Delimiter for char {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.char_indices()
            .find(|(_, c)| self == c)
            .map(|(idx, _)| (idx, idx + self.len_utf8()))
    }
}

impl Delimiter for &str {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.find(self).map(|idx| (idx, idx + self.len()))
    }
}

impl<F: Fn(char) -> bool> Delimiter for F {
    fn find_next(&self, s: &str) -> Option<(usize, usize)> {
        s.find(self).map(|idx| (idx, idx + 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works_char() {
        let s = "a,b,c";
        let expect: Vec<_> = s.split(',').collect();
        let res: Vec<_> = split(s, ',').collect();
        assert_eq!(expect, res);
    }

    #[test]
    fn it_works_str() {
        let s = "apple>>banana>>cherry";
        let expect: Vec<_> = s.split(">>").collect();
        let res: Vec<_> = split(s, ">>").collect();
        assert_eq!(expect, res);
    }

    #[test]
    fn it_works_closure() {
        let s = "a1b2c";
        let expect: Vec<_> = s.split(|c: char| c.is_numeric()).collect();
        let res: Vec<_> = split(s, |c: char| c.is_numeric()).collect();
        assert_eq!(expect, res);
    }
}

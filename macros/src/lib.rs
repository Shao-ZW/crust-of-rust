#[macro_export]
macro_rules! svec {
    ($($element:expr),*) => {{
        const C: usize = $crate::count![@COUNT; $($element),*];
        #[allow(unused_mut)]
        let mut vs = Vec::with_capacity(C);
        $(vs.push($element);)*
        vs
    }};
    ($($element:expr,)*) => {{
        $crate::svec!($($element),*)
    }};
    ($element:expr;$count:expr) => {{
        let mut vs = Vec::new();
        vs.resize($count, $element);
        vs
    }};
}

#[macro_export]
macro_rules! count {
    (@COUNT; $($element:expr),*) => {
        <[()]>::len(&[$($crate::count![@SUBST; $element]),*])
    };
    (@SUBST; $_element:expr) => {
        ()
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let v: Vec<u32> = svec![];
        assert!(v.is_empty());

        let v: Vec<u32> = svec![24, 23,];
        assert_eq!(v.len(), 2);
        assert_eq!(v[0], 24);
        assert_eq!(v[1], 23);

        let v: Vec<u32> = svec![23; 4];
        assert_eq!(v.len(), 4);
        assert_eq!(v[0], 23);
        assert_eq!(v[1], 23);
        assert_eq!(v[2], 23);
        assert_eq!(v[3], 23);
    }
}

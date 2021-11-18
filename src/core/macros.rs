#[macro_export]
macro_rules! vec_iter {
    (  $v:expr, $d: ident) => {
        #[cfg(not(feature = "no_thread"))]
        let $d = $v.par_iter();
        #[cfg(feature = "no_thread")]
        let $d = $v.iter();
    };
}

#
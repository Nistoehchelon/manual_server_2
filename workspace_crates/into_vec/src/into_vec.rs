pub fn into_vec<F, I>(v: Vec<F>) -> Vec<I>
where  
    I: From<F>,
     {
        v.into_iter().map(I::from).collect()
}
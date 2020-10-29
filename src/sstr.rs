use std::collections::HashSet;
use std::sync::Mutex;
use std::marker::PhantomData;

#[derive(Debug)]
struct InnerStringCollection<'a> where Self: 'a {
    set: HashSet<*const str>,
    _data: PhantomData<HashSet<&'a str>>,
}

impl Drop for InnerStringCollection<'_> {
    fn drop(&mut self) {
        for s in self.set.drain() {
            unsafe {
                drop(Box::from_raw(s as *mut str))
            }
        }
    }
}

#[derive(Debug)]
/// Used for extending lifetimes of string slices
/// 
/// ## Example
/// 
/// ```rust
/// # use kondi::sstr::StringCollection;
/// use std::io::stdin;
/// fn readline<'a>(sc: &'a StringCollection<'a>) -> &'a str {
///     let mut s = String::new();
///     stdin().read_line(&mut s).unwrap();
///     sc.str(&s)
/// }
/// 
/// let sc = StringCollection::default();
/// let mut lines = Vec::new();
/// 
/// loop {
///     let s = readline(&sc).trim();
///     lines.push(s);
/// }
/// 
/// for line in lines {
///     println!("{}", line);
/// }
/// ```
/// 
/// This example might as well just use `String`s,
/// but there could be a case where you need a string slice
/// but have to create it anew
pub struct StringCollection<'a> where Self: 'a {
    inner: Mutex<InnerStringCollection<'a>>,
}

impl Default for StringCollection<'_> {
    #[inline(always)]
    fn default() -> Self {
        StringCollection {
            inner: Mutex::new(InnerStringCollection {
                set: HashSet::new(),
                _data: PhantomData
            })
        }
    }
}

impl<'a> StringCollection<'a> where Self: 'a {
    /// Extends the lifetime of the given string
    /// to whatever the lifetime of the collection is
    /// using an inner collection
    /// 
    /// ## Erroneous Example
    /// 
    /// Here is an example of a way, you cannot use this,
    /// see struct documentation for the right way to use it
    /// 
    /// ```compile_fail
    /// # use kondi::sstr::StringCollection;
    /// use std::io::stdin;
    /// fn readline_static() -> &'static str {
    ///     let sc = StringCollection::default();
    ///     let mut s = String::new();
    ///     stdin().read_line(&mut s).unwrap();
    ///     sc.str(&s)
    /// }
    /// ```
    pub fn str<'s: 'a, 'o>(&'a self, s: &'o str) -> &'s str {
        let mut lock = self.inner.lock().unwrap();

        if let Some(&static_str) = lock.set.get(&(s as *const str)) {
            unsafe { &*static_str }
        } else {
            let s: Box<str> = s.into();
            let static_str = Box::into_raw(s) as *const str;
            lock.set.insert(static_str);
            unsafe {&*static_str}
        }
    }
}

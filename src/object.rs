use crate::State;
use std::borrow::Borrow;
use std::fmt::{self, Debug};
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use byteorder::{ByteOrder, BigEndian};

#[derive(Clone)]
struct HashedPointer<T: ?Sized>(Box<T>);

impl HashedPointer<dyn Object> {
    #[inline(always)]
    fn ptr<T>(&self) -> *const T {
        Borrow::<ObjectId<T>>::borrow(self).0
    }
    #[inline(always)]
    fn n(&self) -> usize {
        self.ptr() as *const () as usize
    }
}

impl Debug for HashedPointer<dyn Object> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let o = *Borrow::<ObjectId<()>>::borrow(self);
        o.fmt(fmt)
        // write!(fmt, "hp@{:p}", &*self.0)
    }
}

impl Hash for HashedPointer<dyn Object> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut bytes = [0; ::std::mem::size_of::<usize>()];
        BigEndian::write_u64(&mut bytes, self.n() as u64);
        h.write(&bytes)
    }
}

impl PartialEq<Self> for HashedPointer<dyn Object> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.n() == other.n()
    }
}
impl<T: ?Sized> PartialEq<ObjectId<T>> for HashedPointer<dyn Object> {
    #[inline]
    fn eq(&self, other: &ObjectId<T>) -> bool {
        self.n() == other.n()
    }
}
impl<T> Borrow<ObjectId<T>> for HashedPointer<dyn Object> {
    fn borrow(&self) -> &ObjectId<T> {
        unsafe {
            &*(&*self.0 as *const dyn Object as *const () as *const T as *const ObjectId<T>)
        }
    }
}
impl Eq for HashedPointer<dyn Object> {}
// TODO maybe use an unsafe cell instead, to avoid undefined behaviour from optimisations
// in iter_mut
#[derive(Debug)]
pub struct ObjectSet {
    set: HashSet<HashedPointer<dyn Object>>,
}

impl Default for ObjectSet {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectSet {
    pub fn new() -> Self {
        ObjectSet {
            set: HashSet::new(),
        }
    }
    pub fn add<O: 'static + Object>(&mut self, obj: O) -> ObjectId<O> {
        let hp: HashedPointer<dyn Object> = HashedPointer(Box::new(obj));
        let obj_id = *Borrow::<ObjectId<O>>::borrow(&hp);
        self.set.insert(hp);
        obj_id
    }
    // TODO: maybe return T
    pub fn remove<T>(&mut self, id: ObjectId<T>) -> Option<Box<dyn Object>> {
        self.set.take(&id).map(|HashedPointer(b)| b)
    }
    pub fn get<O: 'static + Object>(&self, id: ObjectId<O>) -> Option<&O> {
        self.set.get(&id).map(|hp| unsafe {
            &*(&*hp.0 as *const (dyn 'static + Object) as *const O)
        })
    }
    pub fn get_mut<'a, O: 'static + Object>(&'a mut self, id: ObjectId<O>) -> Option<&'a mut O> {
        self.set.get(&id).map(|hp| unsafe {
            &mut *(&*hp.0 as *const dyn Object as *mut dyn Object as *mut O)
        })
    }
    pub fn iter(&self) -> impl Iterator<Item=&dyn Object> {
        self.set.iter()
            .map(|hp| {
                &*hp.0
            })
    }
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut dyn Object> {
        self.set.iter()
            // SAFETY:
            // should be fine since this function takes a mutable reference
            // whose lifetime the mutables references generated for the iterator
            .map(|hp| unsafe {
                &mut *(&*hp.0 as *const dyn Object as *mut dyn Object)
            })
    }
    pub fn clear(&mut self) {
        self.set.clear();
    }
}

pub struct ObjectId<T: ?Sized>(*const T);

impl<T: ?Sized> ObjectId<T> {
    #[inline(always)]
    fn ptr(&self) -> *const T {
        self.0 as *const T
    }
    #[inline(always)]
    fn n(&self) -> usize {
        self.ptr() as *const () as usize
    }
}

impl<T: ?Sized> Clone for ObjectId<T> {
    #[inline(always)]
    fn clone(&self) -> ObjectId<T> {
        ObjectId(self.0)
    }
}
impl<T: ?Sized> Copy for ObjectId<T> {}

impl<T: ?Sized> Hash for ObjectId<T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut bytes = [0; ::std::mem::size_of::<usize>()];
        BigEndian::write_u64(&mut bytes, self.n() as u64);
        h.write(&bytes)
    }
}

impl<T: ?Sized, U: ?Sized> PartialEq<ObjectId<U>> for ObjectId<T> {
    fn eq(&self, other: &ObjectId<U>) -> bool {
        self.n() == other.n()
    }
}
impl<T: ?Sized> Eq for ObjectId<T> {}

impl<T: ?Sized> Debug for ObjectId<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "obj@{:p}", self.0)
    }
}

use ggez::{Context, GameResult};
use super::Textures;

pub trait Object {
    fn draw(&self, ctx: &mut Context, texes: &Textures) -> GameResult<()>;
    // Add State ref here
    fn update(&mut self, ctx: &mut Context, state: &mut State, delta: f32);
}

pub mod tex_box;

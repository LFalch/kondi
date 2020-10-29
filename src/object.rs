use std::borrow::Borrow;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};

use byteorder::{ByteOrder, BigEndian};

#[derive(Debug, Clone)]
struct HashedPointer<T: ?Sized>(Box<T>);

impl<T: ?Sized> HashedPointer<T> {
    #[inline(always)]
    fn ptr(&self) -> *const T {
        &*self.0 as *const T
    }
    #[inline(always)]
    fn n(&self) -> usize {
        self.ptr() as *const () as usize
    }
}

impl<T: ?Sized> Hash for HashedPointer<T> {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut bytes = [0; ::std::mem::size_of::<usize>()];
        BigEndian::write_u64(&mut bytes, self.n() as u64);
        h.write(&bytes)
    }
}

impl<T: ?Sized> PartialEq<Self> for HashedPointer<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.n() == other.n()
    }
}
impl PartialEq<ObjectId> for HashedPointer<dyn Object> {
    #[inline]
    fn eq(&self, other: &ObjectId) -> bool {
        self.n() == other.0 as *const () as usize
    }
}
impl Borrow<ObjectId> for HashedPointer<dyn Object> {
    fn borrow(&self) -> &ObjectId {
        // This looks terrifying
        unsafe {
            &*(&((&*self.0) as *const (dyn 'static + Object) as *const () as usize) as *const usize as *const ObjectId)
        }
    }
}
impl<T: ?Sized> Eq for HashedPointer<T> {}
// TODO maybe use an unsafe cell instead, to avoid undefined behaviour from optimisations
// in iter_mut
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
    pub fn add<O: 'static + Object>(&mut self, obj: O) {
        self.set.insert(HashedPointer(Box::new(obj)));
    }
    pub fn remove(&mut self, id: ObjectId) -> Option<Box<dyn Object>> {
        self.set.take(&id).map(|HashedPointer(b)| b)
    }
    pub fn get(&self, id: ObjectId) -> Option<&dyn Object> {
        self.set.get(&id).map(|hp| &*hp.0)
    }
    pub fn iter(&self) -> impl Iterator<Item=&dyn Object> {
        self.set.iter()
            .map(|hp| {
                &*hp.0
            })
    }
    pub fn iter_with_id(&self) -> impl Iterator<Item=(ObjectId, &dyn Object)> {
        self.set.iter()
            .map(|hp| {
                (*hp.borrow(), &*hp.0)
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
}

#[derive(Debug, Copy, Clone)]
pub struct ObjectId(*const ());

impl Hash for ObjectId {
    fn hash<H: Hasher>(&self, h: &mut H) {
        let mut bytes = [0; ::std::mem::size_of::<usize>()];
        BigEndian::write_u64(&mut bytes, self.0 as *const () as u64);
        h.write(&bytes)
    }
}

impl PartialEq for ObjectId {
    fn eq(&self, other: &Self) -> bool {
        self.0 as *const () as u64 == other.0 as *const () as u64
    }
}
impl Eq for ObjectId {}

use ggez::{Context, GameResult};
use super::Textures;

pub trait Object {
    fn draw(&self, ctx: &mut Context, texes: &Textures) -> GameResult<()>;
    // Add State ref here
    fn update(&mut self, delta: f32);
}

pub mod tex_box;
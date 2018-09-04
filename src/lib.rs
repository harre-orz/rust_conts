use std::ptr::{self, NonNull};
use std::marker::PhantomData;

pub trait Pointer<T> : Default {
    fn term(&mut self);

    fn from(&mut self, other: &Self);

    fn set(&mut self, data: &mut T);
    fn as_ref<'a>(&self) -> Option<&'a T>;
    fn as_mut<'a>(&mut self) -> Option<&'a mut T>;
}

impl<T> Pointer<T> for Option<NonNull<T>> {
    fn term(&mut self) {
        *self = None
    }

    fn from(&mut self, other: &Self) {
        *self = *other
    }

    fn set(&mut self, data: &mut T) {
        *self = Some(data.into());
    }

    fn as_ref<'a>(&self) -> Option<&'a T> {
        if let Some(data) = self.as_ref() {
            Some(unsafe { &*data.as_ptr() })
        } else {
            None
        }
    }

    fn as_mut<'a>(&mut self) -> Option<&'a mut T> {
        if let Some(data) = self.as_mut() {
            Some(unsafe { &mut *data.as_ptr() })
        } else {
            None
        }
    }
}



pub trait SListHook<T = Self> : Sized {
    type Pointer: Pointer<T>;
    fn next(&mut self) -> &mut Self::Pointer;
}

pub struct Iter<'a, T: SListHook + 'a> {
    it: NonNull<T::Pointer>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T: SListHook + 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(data) = self.it.as_mut().as_mut() {
                self.it = data.next().into();
                Some(data)
            } else {
                None
            }
        }
    }
}


pub struct IterMut<'a, T: SListHook + 'a> {
    it: NonNull<T::Pointer>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T: SListHook + 'a> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if let Some(data) = self.it.as_mut().as_mut() {
                self.it = data.next().into();
                Some(data)
            } else {
                None
            }
        }
    }
}

impl<'a, T: SListHook + 'a> From<IterMut<'a, T>> for Iter<'a, T> {
    fn from(it: IterMut<'a, T>) -> Self {
        Iter {
            it: it.it,
            _marker: PhantomData,
        }
    }
}


pub struct SList<T: SListHook<T>> {
    size: usize,
    head: T::Pointer,
    _marker: PhantomData<T>,
}

impl<T: SListHook<T>> SList<T> {
    pub fn new() -> Self {
        SList {
            size: 0,
            head: Default::default(),
            _marker: PhantomData,
        }
    }

    pub fn size(&self) -> usize {
        self.size
    }

    pub fn iter<'a>(&'a self) -> Iter<'a, T> {
        Iter {
            it: (&self.head).into(),
            _marker: PhantomData,
        }
    }

    pub fn iter_mut<'a>(&'a self) -> IterMut<'a, T> {
        IterMut {
            it: (&self.head).into(),
            _marker: PhantomData,
        }
    }

    pub fn push(&mut self, data: &mut T) {
        data.next().term();
        self.head.set(data);
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<&mut T> {
        if let Some(data) = self.head.as_mut() {
            self.head.from(data.next());
            self.size -= 1;
            return Some(data)
        } else {
            return None
        }
    }
}


struct Wrapped<T, P: SListHook<Wrapped<T, P>>> {
    next: P::Pointer,
    t: T,
}

impl<T, P: SListHook<Self>> SListHook<Self> for Wrapped<T, P> {
    type Pointer = P::Pointer;

    fn next(&mut self) -> &mut Self::Pointer {
        &mut self.next
    }
}

impl<T, A: Allocator> SListHook<Wrapped<T, Self>> for SList2<T, A> {
    type Pointer = Option<NonNull<Wrapped<T, Self>>>;

    fn next(&mut self) -> &mut Self::Pointer {
        &mut self.inner.head
    }
}

pub trait Allocator {
}

pub struct SList2<T, A: Allocator> {
    inner: SList<Wrapped<T, SList2<T, A>>>,
    alloc: A,
}

#[test]
fn test_slist() {
    struct MyObject {
        next: Option<NonNull<MyObject>>,
        x: i32,
    }

    impl SListHook for MyObject {
        type Pointer = Option<NonNull<Self>>;
        fn next(&mut self) -> &mut Option<NonNull<MyObject>> { &mut self.next }
    }

    let mut x: SList<MyObject> = SList::new();
    let mut y = MyObject {
        x: 100,
        next: Default::default(),
    };
    x.push(&mut y);
    x.pop().unwrap();
}

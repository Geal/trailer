use std::{alloc, mem, ops::{Deref, DerefMut, Drop}, default::Default, marker::PhantomData};

struct Trailer<T> {
    //pub inner: Inner,
    pub ptr: *mut u8,
    pub size: usize,
    phantom: PhantomData<T>,
}

impl<T:Default> Trailer<T> {
    fn new(capacity: usize) -> Trailer<T> {
        let size = ::std::mem::size_of::<T>() + capacity;
        let align = mem::align_of::<u8>();
        let layout = alloc::Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc::alloc(layout) };

        unsafe {
            let mut inner = ptr as *mut T;
            *inner = T::default();
        }

        Trailer { ptr, size, phantom: PhantomData }
    }

    fn bytes(&self) -> &[u8] {
        unsafe {
            ::std::slice::from_raw_parts(
                self.ptr.offset(::std::mem::size_of::<T>() as isize),
                self.size - ::std::mem::size_of::<T>(),
            )
        }
    }

    fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                self.ptr.offset(::std::mem::size_of::<T>() as isize),
                self.size - ::std::mem::size_of::<T>(),
            )
        }
    }
}

impl<T> Drop for Trailer<T> {
    fn drop(&mut self) {
        //(&mut *(self.ptr as *mut T)).drop();
        let align = mem::align_of::<u8>();
        let layout = alloc::Layout::from_size_align(self.size, align).unwrap();
        unsafe { alloc::dealloc(self.ptr, layout) };
    }
}

impl<T> Deref for Trailer<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*(self.ptr as *const T) }
    }
}

impl<T> DerefMut for Trailer<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[derive(Debug, Default)]
  struct Inner {
    field1: usize,
    field2: bool,
  }

  type Data = Trailer<Inner>;

    #[test]
    fn trailer() {
        let mut a = Data::new(100);
        a.field1 = 12345;
        a.field2 = true;

        a.bytes_mut()[0] = 1;
        a.bytes_mut()[1] = 2;
        a.bytes_mut()[2] = 3;
        a.bytes_mut()[3] = 4;

        println!("Inner: {:?}", *a);
        println!("bytes: {:?}", a.bytes());
        println!("raw bytes: {:?}", unsafe {
            ::std::slice::from_raw_parts(a.ptr, a.size)
        });
        panic!();
    }
}

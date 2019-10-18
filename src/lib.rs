use std::{
    alloc,
    default::Default,
    marker::PhantomData,
    mem::{self, MaybeUninit},
    ops::{Deref, DerefMut, Drop},
    ptr,
};

struct Trailer<T> {
    pub ptr: *mut u8,
    pub size: usize,
    phantom: PhantomData<T>,
}

impl<T: Default> Trailer<T> {
    fn new(capacity: usize) -> Trailer<T> {
        unsafe {
            let mut trailer = Trailer::allocate(capacity);
            let mut inner = trailer.ptr as *mut T;
            let previous = mem::replace((&mut *(trailer.ptr as *mut T)), T::default());
            mem::forget(previous);

            trailer
        }
    }
}

impl<T: Copy> Trailer<T> {
    fn from(t: T, capacity: usize) -> Trailer<T> {
        unsafe {
            let mut trailer = Trailer::allocate(capacity);
            let mut inner = trailer.ptr as *mut T;
            *inner = t;

            trailer
        }
    }
}

impl<T> Trailer<T> {
    unsafe fn allocate(capacity: usize) -> Trailer<T> {
        let size = ::std::mem::size_of::<T>() + capacity;
        let align = mem::align_of::<u8>();
        let layout = alloc::Layout::from_size_align(size, align).unwrap();
        let ptr = unsafe { alloc::alloc(layout) };

        Trailer {
            ptr,
            size,
            phantom: PhantomData,
        }
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
        unsafe { ptr::drop_in_place(self.ptr as *mut T) };
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

    #[test]
    fn default() {
        #[derive(Debug, Default)]
        struct Inner {
            field1: usize,
            field2: bool,
        }

        impl Drop for Inner {
            fn drop(&mut self) {
                println!("dropping Inner instance");
            }
        }

        type Data = Trailer<Inner>;
        // wrapper to test that the Inner instance is dropped
        {
            println!("will create");
            let mut a = Data::new(100);
            println!("created");
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
        }
        assert_eq!(::std::mem::size_of::<Data>(), 16);
        assert_eq!(::std::mem::align_of::<Data>(), 8);
        panic!();
    }

    #[test]
    fn copy() {
        #[derive(Debug, Clone, Copy)]
        struct Inner {
            field1: usize,
            field2: bool,
        }

        type Data = Trailer<Inner>;

        let inner = Inner {
            field1: 5678,
            field2: true,
        };
        println!("will create");
        let mut a = Data::from(inner, 100);
        println!("created");
        //a.field1 = 12345;
        //a.field2 = true;

        a.bytes_mut()[0] = 1;
        a.bytes_mut()[1] = 2;
        a.bytes_mut()[2] = 3;
        a.bytes_mut()[3] = 4;

        println!("Inner: {:?}", *a);
        println!("bytes: {:?}", a.bytes());
        println!("raw bytes: {:?}", unsafe {
            ::std::slice::from_raw_parts(a.ptr, a.size)
        });
        assert_eq!(::std::mem::size_of::<Data>(), 16);
        assert_eq!(::std::mem::align_of::<Data>(), 8);
        panic!();
    }
}

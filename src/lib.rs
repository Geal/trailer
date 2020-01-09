use std::{
    alloc,
    default::Default,
    marker::PhantomData,
    mem,
    ops::{Deref, DerefMut, Drop},
    ptr, slice,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Trailer<T> {
    ptr: *mut u8,
    size: usize,
    phantom: PhantomData<T>,
}

impl<T: Default> Trailer<T> {
    pub fn new(capacity: usize) -> Trailer<T> {
        unsafe {
            let trailer = Trailer::allocate(capacity);
            let ptr = trailer.ptr as *mut T;
            ptr.write(T::default());
            trailer
        }
    }
}

impl<T: Copy> Trailer<T> {
    pub fn from(t: T, capacity: usize) -> Trailer<T> {
        unsafe {
            let trailer = Trailer::allocate(capacity);
            let ptr = trailer.ptr as *mut T;
            ptr.write(t);

            trailer
        }
    }
}

impl<T> Trailer<T> {
    unsafe fn allocate(capacity: usize) -> Trailer<T> {
        let size = mem::size_of::<T>() + capacity;
        let align = mem::align_of::<T>();
        let layout = alloc::Layout::from_size_align(size, align).unwrap();
        let ptr = alloc::alloc_zeroed(layout);

        Trailer {
            ptr,
            size,
            phantom: PhantomData,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        unsafe {
            slice::from_raw_parts(
                self.ptr.add(mem::size_of::<T>()),
                self.size - mem::size_of::<T>(),
            )
        }
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        unsafe {
            ::std::slice::from_raw_parts_mut(
                self.ptr.add(mem::size_of::<T>()),
                self.size - mem::size_of::<T>(),
            )
        }
    }
}

impl<T> Drop for Trailer<T> {
    fn drop(&mut self) {
        unsafe { ptr::drop_in_place(self.ptr as *mut T) };
        let align = mem::align_of::<T>();
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
            let raw = unsafe { ::std::slice::from_raw_parts(a.ptr, a.size) };
            println!("raw bytes: {:?}", raw);
            assert_eq!(&raw[..20], &vec![57u8, 48, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4][..]);
        }
        assert_eq!(::std::mem::size_of::<Data>(), 16);
        assert_eq!(::std::mem::align_of::<Data>(), 8);
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
        let raw = unsafe { ::std::slice::from_raw_parts(a.ptr, a.size) };
        println!("raw bytes: {:?}", raw);
        assert_eq!(&raw[..20], &vec![46u8, 22, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 4][..]);

        assert_eq!(::std::mem::size_of::<Data>(), 16);
        assert_eq!(::std::mem::align_of::<Data>(), 8);
    }
}

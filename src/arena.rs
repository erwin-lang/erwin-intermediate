use std::{cell::RefCell, iter::repeat_n, marker::PhantomData, slice::from_raw_parts};

pub(crate) struct Arena<'a> {
    sized_storage: RefCell<Vec<Vec<u8>>>,
    dyn_storage: RefCell<Vec<Box<dyn Erased + 'a>>>,
    _marker: PhantomData<&'a ()>,
}

impl<'a> Arena<'a> {
    const CHUNK_SIZE: usize = 1024 * 1024;

    pub(crate) fn new() -> Self {
        Self {
            sized_storage: RefCell::new(vec![Vec::with_capacity(Self::CHUNK_SIZE)]),
            dyn_storage: RefCell::new(Vec::new()),
            _marker: PhantomData,
        }
    }

    pub(crate) fn sized_alloc<T: Copy + 'a>(&self, val: T) -> &'a T {
        let mut chunks = self.sized_storage.borrow_mut();
        let mut current_chunk = chunks.last_mut().unwrap();

        let align = align_of::<T>();
        let size = size_of::<T>();
        let padding = (align - (current_chunk.len() % align)) % align;

        if current_chunk.len() + padding + size > current_chunk.capacity() {
            chunks.push(Vec::with_capacity(Self::CHUNK_SIZE.max(size + align)));
            current_chunk = chunks.last_mut().unwrap();
        }

        if padding > 0 && !current_chunk.is_empty() {
            current_chunk.extend(repeat_n(0, padding));
        }

        let start_offset = current_chunk.len();

        let bytes = unsafe { from_raw_parts(&val as *const T as *const u8, size) };
        current_chunk.extend_from_slice(bytes);

        let ptr = unsafe { current_chunk.as_ptr().add(start_offset) as *const T };

        unsafe { &*ptr }
    }

    pub(crate) fn dyn_alloc<T: 'a>(&self, val: T) -> &'a T {
        let boxed = Box::new(val);
        let ptr = &*boxed as *const T;

        self.dyn_storage.borrow_mut().push(boxed);

        unsafe { &*ptr }
    }

    pub(crate) fn contains_ptr<T>(&self, ptr: *const T) -> bool {
        let p = ptr as usize;

        let chunks = self.sized_storage.borrow();
        for chunk in chunks.iter() {
            if !chunk.is_empty() {
                let start = chunk.as_ptr() as usize;
                let end = start + chunk.len();

                if p >= start && p < end {
                    return true;
                }
            }
        }

        self.dyn_storage.borrow().iter().any(|boxed| {
            let unboxed = boxed.as_ref();
            let start = unboxed as *const dyn Erased as *const u8 as usize;
            let end = start + size_of_val(unboxed);

            p >= start && p < end
        })
    }
}

trait Erased {}
impl<T> Erased for T {}

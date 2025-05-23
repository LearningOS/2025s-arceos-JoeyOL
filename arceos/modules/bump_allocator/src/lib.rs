#![no_std]

use core::{alloc::Layout, cmp::max, ptr::NonNull};

use allocator::{AllocError, AllocResult, BaseAllocator, ByteAllocator, PageAllocator};

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const PAGE_SIZE: usize> {
    start: usize,
    end: usize,
    b_pos: usize,
    p_pos: usize,
    count: usize,
}

impl<const PAGE_SIZE: usize> EarlyAllocator<PAGE_SIZE> {
    pub const fn new() -> Self {
        Self {
            start: 0,
            end: 0,
            b_pos: 0,
            p_pos: 0,
            count: 0,
        }
    }
}

impl<const PAGE_SIZE: usize> BaseAllocator for EarlyAllocator<PAGE_SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.end = start + size;
        self.b_pos = start;
        self.p_pos = self.end;
        self.count = 0;
    }

    fn add_memory(&mut self, start: usize, size: usize) -> AllocResult {
        let _ = size;
        let _ = start;
        panic!("EarlyAllocator does not support add_memory");
    }
}

impl<const PAGE_SIZE: usize> ByteAllocator for EarlyAllocator<PAGE_SIZE> {
    fn alloc(&mut self, layout: Layout) -> AllocResult<NonNull<u8>> {
        let size = layout.size();
        let align = layout.align();
        let mut pos = self.b_pos;
        if pos % align != 0 {
            pos = (pos + align - 1) & !(align - 1);
        }
        if pos + size > self.p_pos {
            return Err(AllocError::NoMemory);
        }
        let ret;
        unsafe {
            ret = NonNull::new_unchecked(pos as *mut u8);
        }
        self.b_pos = pos + size;
        self.count += 1;
        Ok(ret)
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: Layout) {
        let _ = pos;
        let _ = layout;
        self.count = max(0, self.count-1);
        if self.count == 0 {
            self.b_pos = self.start;
        }
    }

    fn total_bytes(&self) -> usize {
        self.end - self.start
    }

    fn used_bytes(&self) -> usize {
        self.b_pos - self.start 
    }

    fn available_bytes(&self) -> usize {
        self.p_pos - self.b_pos
    }
}

impl<const PAGE_SIZE: usize> PageAllocator for EarlyAllocator<PAGE_SIZE> {
    const PAGE_SIZE: usize = PAGE_SIZE;

    fn alloc_pages(&mut self, num_pages: usize, align_pow2: usize) -> AllocResult<usize> {
        let size = num_pages * PAGE_SIZE;
        let mut pos = self.p_pos;
        if pos % align_pow2 != 0 {
            pos = (pos + align_pow2 - 1) & !(align_pow2 - 1);
        }

        if pos - size < self.b_pos {
            return Err(AllocError::NoMemory);
        }
        let ret = pos - size;
        self.p_pos = ret;
        Ok(ret)
    }

    fn dealloc_pages(&mut self, pos: usize, num_pages: usize) {
        let _ = pos;
        let _ = num_pages;
        panic!("EarlyAllocator does not support dealloc_pages");
    }

    fn total_pages(&self) -> usize {
        (self.end - self.start) / PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        (self.end - self.p_pos) / PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        (self.p_pos - self.b_pos) / PAGE_SIZE
    }
}

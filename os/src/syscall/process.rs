//! Process management syscalls

use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, PageTable, VirtAddr},
    task::{
        calltime, change_program_brk, current_user_token, exit_current_and_run_next, mmap, munmap,
        suspend_current_and_run_next,
    },
    timer::get_time,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    // trace!("kernel: sys_get_time");
    let us = get_time();
    let time = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let len = core::mem::size_of::<TimeVal>();
    let buffers = translated_byte_buffer(current_user_token(), ts as *const u8, len);
    let src = unsafe { core::slice::from_raw_parts(&time as *const TimeVal as *const u8, len) };
    let buffers_len = buffers.concat().len();
    let src_len = src.len();

    let mut src_iter = src.iter();
    for buf in buffers.into_iter() {
        for b in buf.iter_mut() {
            if let Some(s) = src_iter.next() {
                *b = *s;
            }
        }
    }
    if buffers_len == src_len {
        0
    } else {
        -1
    }
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    // trace!("kernel: sys_trace");

    let token = current_user_token();
    let page_teble = PageTable::from_token(token);
    let va = VirtAddr::from(id);

    const MAX_VA: usize = (1 << 38) - 1;
    const MIN_VA: usize = (!0) << 38;
    if !(id <= MAX_VA || id >= MIN_VA) {
        return -1;
    }

    match trace_request {
        0 => match page_teble.translate(va.floor()) {
            Some(pte) => {
                if pte.readable() {
                    let buffers = translated_byte_buffer(token, id as *const u8, 1);
                    buffers[0][0] as isize
                } else {
                    -1
                }
            }
            None => -1,
        },
        1 => match page_teble.translate(va.floor()) {
            Some(pte) => {
                if pte.writable() {
                    let mut buffers = translated_byte_buffer(token, id as *mut u8, 1);
                    buffers[0][0] = data as u8;
                    0
                } else {
                    -1
                }
            }
            None => -1,
        },
        2 => calltime(id),
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    // trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");

    if len == 0 {
        return -1;
    }
    if (start & (PAGE_SIZE - 1) != 0) || (port & !0x7 != 0) || (port & 0x7 == 0) {
        return -1;
    }
    mmap(start, len, port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    // trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");

    // page.page_offset != 0 就是没有对齐
    if len == 0 || (start & (PAGE_SIZE - 1)) != 0 || (len & (PAGE_SIZE - 1)) != 0 {
        return -1;
    }
    munmap(start, len)
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}

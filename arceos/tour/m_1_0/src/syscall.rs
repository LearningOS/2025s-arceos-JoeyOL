#![allow(dead_code)]

use axhal::arch::TrapFrame;
use axhal::trap::{register_trap_handler, SYSCALL, PAGE_FAULT};
use axhal::paging::MappingFlags;
use axhal::mem::VirtAddr;
use axerrno::LinuxError;
use axtask::TaskExtRef;

const SYS_EXIT: usize = 93;

#[register_trap_handler(SYSCALL)]
fn handle_syscall(tf: &TrapFrame, syscall_num: usize) -> isize {
    ax_println!("handle_syscall ...");
    let ret = match syscall_num {
        SYS_EXIT => {
            ax_println!("[SYS_EXIT]: process is exiting ..");
            axtask::exit(tf.arg0() as _)
        },
        _ => {
            ax_println!("Unimplemented syscall: {}", syscall_num);
            -LinuxError::ENOSYS.code() as _
        }
    };
    ret
}

#[register_trap_handler(PAGE_FAULT)]
fn handle_page_fault(va: VirtAddr, flag: MappingFlags, is_user: bool) -> bool {
    if is_user {
        if !axtask::current().task_ext().aspace.lock().handle_page_fault(va, flag) {
            ax_println!("{}: segmentation fault, exit!", axtask::current().id_name());
            axtask::exit(-1);
            panic!("unreachable code!");
        }
        else {
            ax_println!("{}: handle page fault OK!", axtask::current().id_name());
        }
        true
    } else {
        false
    }
}
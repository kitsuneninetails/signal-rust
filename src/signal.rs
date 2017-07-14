use std::mem::{transmute, uninitialized};

use libc;

pub use libc::{SIGINT, SIGTERM, SIGUSR1, SIGUSR2};
pub use libc::{SIGHUP, SIGQUIT, SIGPIPE, SIGALRM, SIGTRAP};
pub use libc::c_int as int;

pub fn signal(signum: int, sig_handler: fn(int), flags: Option<int>) {
    let int_sig_handler: libc::size_t = unsafe { transmute(sig_handler) };
    signal_internal(signum, int_sig_handler, flags)
}

fn signal_internal(signum: int, sig_handler: libc::size_t, flags: Option<int>) {
    let mut sigset: libc::sigset_t = unsafe { uninitialized() };
    let _ = unsafe { libc::sigemptyset(&mut sigset as *mut libc::sigset_t) };
    
    let mut siga = unsafe { uninitialized::<libc::sigaction>() };
    let mut oldact = unsafe { uninitialized::<libc::sigaction>() };
    
    siga.sa_sigaction = sig_handler;
    siga.sa_mask = sigset;
    siga.sa_flags = flags.map(|x| { x - libc::SA_SIGINFO }).unwrap_or(libc::SA_ONSTACK | libc::SA_RESTART);
    siga.sa_restorer = None;
    
    unsafe { libc::sigaction(signum, &siga as *const libc::sigaction, &mut oldact as *mut libc::sigaction) };
}

pub fn default(signum: int) {
    let int_sig_handler: libc::size_t = unsafe { transmute(libc::SIG_DFL) };
    signal_internal(signum, int_sig_handler, None)
}

pub fn ignore(signum: int) {
    let int_sig_handler: libc::size_t = unsafe { transmute(libc::SIG_IGN) };
    signal_internal(signum, int_sig_handler, None)
}

pub fn kill(signal: int) {
    unsafe { libc::kill(libc::getpid(), signal) };
}

#[cfg(test)]
mod tests {
    use super::*;
    use channel::SyncChannel;
    
    lazy_static! {
        static ref TERM_CH: SyncChannel<()> = SyncChannel::new();
    }
    
    fn test_handler(_: int) {
        TERM_CH.send(()).unwrap();
    }
    
    #[test]
    fn test_signals() {
        signal(SIGUSR1, test_handler, None);
        
        kill(SIGUSR1);
        TERM_CH.recv().unwrap();
        
        default(SIGUSR1);
        
        ignore(SIGUSR1);
        
        kill(SIGUSR1);
        
        signal(SIGUSR1, test_handler, None);
        signal(SIGUSR2, test_handler, None);
        
        kill(SIGUSR1);
        TERM_CH.recv().unwrap();
        
        kill(SIGUSR2);
        TERM_CH.recv().unwrap();
    }
    
}

use std::sync::mpsc::{Sender, Receiver, channel, TryRecvError};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum ChannelError {
    SyncError(String),
    SendError,
    RecvError,
    
}

pub struct SyncChannel<T> {
    _s: Arc<Mutex<Sender<T>>>,
    _r: Arc<Mutex<Receiver<T>>>
}

impl<T> SyncChannel<T> {
    pub fn new() -> Self {
        let (s, r) = channel::<T>();
        SyncChannel { _s: Arc::new(Mutex::new(s)), _r: Arc::new(Mutex::new(r)) }
    }
    
    pub fn send(&self, data: T) -> Result<(), ChannelError> {
        self._s.lock()
            .map_err(|e| { ChannelError::SyncError(format!("Mutex poisoned: {:?}", e)) })?
            .send(data)
            .map_err(|_| { ChannelError::SendError })
    }
    
    pub fn recv(&self) -> Result<T, ChannelError> {
        self._r.lock()
            .map_err(|e| { ChannelError::SyncError(format!("Mutex poisoned: {:?}", e)) })?
            .recv()
            .map_err(|_| { ChannelError::RecvError })
    }
    
    pub fn try_recv(&self) -> Result<Option<T>, ChannelError> {
        match self._r.lock()
            .map_err(|e| { ChannelError::SyncError(format!("Mutex poisoned: {:?}", e)) })?
            .try_recv() {
            
            Err(TryRecvError::Empty) => Ok(None),
            Err(_) => Err(ChannelError::RecvError),
            Ok(d) => Ok(Some(d))
        }
        
    }
}

impl<T> Clone for SyncChannel<T> {
    fn clone(&self) -> Self {
        SyncChannel {
            _r: self._r.clone(),
            _s: self._s.clone(),
        }
    }
}
unsafe impl<T> Send for SyncChannel<T> {}
unsafe impl<T> Sync for SyncChannel<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::atomic::{AtomicBool, Ordering};
    
    #[test]
    fn test_channels() {
        let ch: SyncChannel<i32> = SyncChannel::new();
        let ch_clone = ch.clone();
        thread::spawn(move || {
            ch_clone.send(3)
        });
        assert_eq!(ch.recv().unwrap(), 3);
    }
    
    #[test]
    fn test_channels_try_recv() {
        let ch: SyncChannel<i32> = SyncChannel::new();
        let ch_clone = ch.clone();
        let switch = Arc::new(AtomicBool::new(false));
        let sw_clone = switch.clone();
        assert_eq!(ch.try_recv().unwrap(), None);
        
        thread::spawn(move || {
            ch_clone.send(3).unwrap();
            sw_clone.store(true, Ordering::Relaxed);
        });
        loop {
            if switch.load(Ordering::Relaxed) { break; }
        }
        assert_eq!(ch.try_recv().unwrap(), Some(3));
    }
}
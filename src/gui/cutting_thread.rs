use std::{
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use crate::cut::{cut_song, CutInfo};

struct CuttingThread {
    pub to_cut: Vec<CutInfo>,
    receiver: Receiver<CutInfo>,
}

impl CuttingThread {
    pub fn cutting_loop(&mut self) {
        loop {
            if let Some(info) = self.to_cut.pop() {
                cut_song(&info).unwrap();
            }
            if let Ok(received) = self.receiver.recv() {
                self.to_cut.push(received);
            }
        }
    }
}

impl CuttingThread {
    fn new(receiver: Receiver<CutInfo>) -> Self {
        CuttingThread {
            to_cut: vec![],
            receiver,
        }
    }
}

pub struct CuttingThreadHandle {
    _handle: JoinHandle<()>,
    sender: Sender<CutInfo>,
}

impl Default for CuttingThreadHandle {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let handle = thread::spawn(|| CuttingThread::new(receiver).cutting_loop());
        CuttingThreadHandle {
            _handle: handle,
            sender,
        }
    }
}

impl CuttingThreadHandle {
    pub fn send_cut_infos(&self, cut_infos: Vec<CutInfo>) {
        for cut_info in cut_infos {
            self.sender.send(cut_info).unwrap();
        }
    }
}

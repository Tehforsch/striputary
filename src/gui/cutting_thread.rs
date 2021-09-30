use std::{sync::mpsc::{channel, Receiver, Sender}, thread::{self, JoinHandle}, time::Duration};

use crate::{cut::{cut_song, CutInfo}, song::Song};

struct CuttingThread {
    pub to_cut: Vec<CutInfo>,
    receiver: Receiver<CutInfo>,
    song_sender: Sender<Song>,
}

impl CuttingThread {
    pub fn cutting_loop(&mut self) {
        loop {
            if let Some(info) = self.to_cut.pop() {
                cut_song(&info).unwrap();
                self.song_sender.send(info.song).unwrap();
            }
            if let Ok(received) = self.receiver.recv() {
                self.to_cut.push(received);
            }
        }
    }
}

impl CuttingThread {
    fn new(receiver: Receiver<CutInfo>, song_sender: Sender<Song>) -> Self {
        CuttingThread {
            to_cut: vec![],
            receiver,
            song_sender,
        }
    }
}

pub struct CuttingThreadHandle {
    _handle: JoinHandle<()>,
    sender: Sender<CutInfo>,
    song_receiver: Receiver<Song>,
    cut_songs: Vec<Song>,
}

impl Default for CuttingThreadHandle {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let (song_sender, song_receiver) = channel();
        let handle = thread::spawn(|| CuttingThread::new(receiver, song_sender).cutting_loop());
        CuttingThreadHandle {
            _handle: handle,
            sender,
            song_receiver,
            cut_songs: vec![],
        }
    }
}

impl CuttingThreadHandle {
    pub fn send_cut_infos(&self, cut_infos: Vec<CutInfo>) {
        for cut_info in cut_infos {
            self.sender.send(cut_info).unwrap();
        }
    }

    pub fn get_cut_songs(&mut self) -> &Vec<Song> {
        if let Ok(received) = self.song_receiver.recv_timeout(Duration::from_millis(5)) {
            self.cut_songs.push(dbg!(received));
        }
        &self.cut_songs
    }
}

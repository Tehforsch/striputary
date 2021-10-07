use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::Sender;
use std::thread::JoinHandle;
use std::thread::{self};

use crate::config;
use crate::cut::cut_song;
use crate::cut::CutInfo;
use crate::data_stream::DataStream;
use crate::song::Song;

struct CuttingThread {
    pub to_cut: DataStream<CutInfo>,
    song_sender: Sender<Song>,
}

impl CuttingThread {
    pub fn cutting_loop(&mut self) {
        loop {
            if let Some(info) = self.to_cut.get_data_mut().pop() {
                cut_song(&info).unwrap();
                self.song_sender.send(info.song).unwrap();
            }
            self.to_cut.update(config::RECV_CUT_INFO_TIMEOUT);
        }
    }
}

impl CuttingThread {
    fn new(receiver: Receiver<CutInfo>, song_sender: Sender<Song>) -> Self {
        CuttingThread {
            to_cut: DataStream::new(receiver),
            song_sender,
        }
    }
}

pub struct CuttingThreadHandle {
    _handle: JoinHandle<()>,
    sender: Sender<CutInfo>,
    cut_songs: DataStream<Song>,
}

impl Default for CuttingThreadHandle {
    fn default() -> Self {
        let (sender, receiver) = channel();
        let (song_sender, song_receiver) = channel();
        let handle = thread::spawn(|| CuttingThread::new(receiver, song_sender).cutting_loop());
        CuttingThreadHandle {
            _handle: handle,
            sender,
            cut_songs: DataStream::new(song_receiver),
        }
    }
}

impl CuttingThreadHandle {
    pub fn send_cut_infos(&self, cut_infos: Vec<CutInfo>) {
        for cut_info in cut_infos {
            self.sender.send(cut_info).unwrap();
        }
    }

    pub fn get_cut_songs(&mut self) -> &[Song] {
        self.cut_songs.update(config::RECV_CUT_SONG_TIMEOUT);
        &self.cut_songs.get_data()
    }
}

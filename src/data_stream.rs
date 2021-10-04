use std::{sync::mpsc::Receiver, time::Duration};

pub struct DataStream<T> {
    receiver: Receiver<T>,
    data: Vec<T>,
}

impl<T> DataStream<T> {
    pub fn new(receiver: Receiver<T>) -> Self {
        DataStream {
            receiver,
            data: vec![],
        }
    }

    pub fn update(&mut self, timeout: Duration) {
        let new_data = self.receiver.recv_timeout(timeout).ok();
        if let Some(new_data) = new_data {
            self.data.push(new_data);
        }
    }

    pub fn get_data(&self) -> &[T] {
        &self.data
    }
}

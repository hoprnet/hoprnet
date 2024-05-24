use dashmap::mapref::entry::Entry;
use dashmap::DashMap;
use futures::Stream;
use std::sync::atomic::{AtomicU16, AtomicU32, Ordering};
use std::sync::OnceLock;

pub struct Frame<'a> {
    seq_num: u16,
    seq_len: u16,
    seq_idx: u16,
    data: &'a [u8],
}

impl<'a> Frame<'a> {
    pub fn seq_id(&self) -> u32 {
        (self.seq_num as u32) << 16 | self.seq_len as u32
    }

    pub fn segment(data: &'a [u8], mtu: usize, seq_num: u16) -> Vec<Frame<'a>> {
        let chunks = data.chunks(mtu);
        assert!(chunks.len() < u16::MAX as usize, "data too long");

        let seq_len = chunks.len() as u16;

        chunks
            .enumerate()
            .map(|(idx, data)| Self {
                seq_num,
                seq_len,
                seq_idx: idx as u16,
                data,
            })
            .collect()
    }

    pub fn segment_iter<T: IntoIterator<Item = &'a [u8]>>(data: T, mtu: usize, first_seq_num: u16) -> Vec<Frame<'a>> {
        data.into_iter()
            .take((u16::MAX - first_seq_num) as usize)
            .enumerate()
            .flat_map(|(seq, p)| Frame::segment(p, mtu, first_seq_num + seq as u16))
            .collect::<Vec<_>>()
    }
}

struct FrameTable<'a> {
    segments: Vec<OnceLock<&'a [u8]>>,
    missing: AtomicU16,
}

impl<'a> FrameTable<'a> {
    fn new(initial_frame: Frame<'a>) -> Self {
        Self {
            segments: (0..initial_frame.seq_len)
                .map(|i| {
                    if i == initial_frame.seq_idx {
                        OnceLock::from(initial_frame.data)
                    } else {
                        OnceLock::new()
                    }
                })
                .collect(),
            missing: AtomicU16::new(initial_frame.seq_len - 1),
        }
    }

    fn put(&self, frame: Frame<'a>) -> u16 {
        if !self.is_complete() {
            if self.segments[frame.seq_idx as usize].set(frame.data).is_ok() {
                self.missing.fetch_sub(1, Ordering::SeqCst);
            }
            self.missing.load(Ordering::SeqCst)
        } else {
            0
        }
    }

    fn is_complete(&self) -> bool {
        self.missing.load(Ordering::SeqCst) == 0
    }

    fn reassemble(self) -> Box<[u8]> {
        assert!(self.is_complete(), "missing frames");
        self.segments
            .into_iter()
            .map(|lock| lock.into_inner().unwrap())
            .collect::<Vec<&[u8]>>()
            .concat()
            .into_boxed_slice()
    }
}

pub struct FrameReassembler<'a> {
    sequences: DashMap<u32, FrameTable<'a>>,
    lowest_seq: AtomicU32,
    complete_send: async_channel::Sender<(u16, Box<[u8]>)>,
    complete_recv: async_channel::Receiver<(u16, Box<[u8]>)>,
}

impl<'a> FrameReassembler<'a> {
    pub fn new() -> Self {
        let (complete_send, complete_recv) = async_channel::unbounded();
        Self {
            sequences: DashMap::new(),
            lowest_seq: Default::default(),
            complete_send,
            complete_recv,
        }
    }

    pub fn add_frame(&self, frame: Frame<'a>) {
        let seq_id = frame.seq_id();

        match self.sequences.entry(seq_id) {
            Entry::Occupied(e) => {
                e.get().put(frame);
            }
            Entry::Vacant(v) => {
                v.insert(FrameTable::new(frame));
                self.lowest_seq.fetch_min(seq_id, Ordering::SeqCst);
            }
        }

        if self
            .sequences
            .get(&self.lowest_seq.load(Ordering::SeqCst))
            .is_some_and(|t| t.is_complete())
        {
            if let Some((seq_id, table)) = self.sequences.remove(&seq_id) {
                let seq_num = (seq_id & 0xffff0000) >> 16;
                let _ = self.complete_send.try_send((seq_num as u16, table.reassemble()));
            }
        }
    }

    pub fn stream(&self) -> impl Stream<Item = (u16, Box<[u8]>)> {
        self.complete_recv.clone()
    }

    pub fn close(self) {
        self.complete_send.close();
        self.complete_recv.close();
    }
}

#[cfg(test)]
mod tests {
    use crate::frame::{Frame, FrameReassembler};
    use futures::StreamExt;
    use rand::prelude::SliceRandom;
    use rand::thread_rng;
    use rayon::prelude::*;

    #[async_std::test]
    async fn test_shuffled_randomized() {
        let packets = (0..65_535)
            .into_par_iter()
            .map(|_| hopr_crypto_random::random_bytes::<500>())
            .collect::<Vec<_>>();

        let mut fragments = Frame::segment_iter(packets.iter().map(|s| s.as_slice()), 10, 0);

        let mut rng = thread_rng();
        fragments.shuffle(&mut rng);

        let reassembler = FrameReassembler::new();

        let stream = reassembler.stream();
        let jh = async_std::task::spawn(async move { stream.collect::<Vec<_>>().await });

        fragments.into_iter().for_each(|f| reassembler.add_frame(f));
        reassembler.close();

        jh.await.into_par_iter().for_each(|(seq_num, reassembled)| {
            assert_eq!(reassembled.as_ref(), packets[seq_num as usize].as_ref());
        });
    }

    #[async_std::test]
    async fn test_shuffled_randomized_parallel() {
        let packets = (0..65_535)
            .into_par_iter()
            .map(|_| hopr_crypto_random::random_bytes::<500>())
            .collect::<Vec<_>>();

        let mut fragments = Frame::segment_iter(packets.iter().map(|p| p.as_slice()), 10, 0);

        let mut rng = thread_rng();
        fragments.shuffle(&mut rng);

        let reassembler = FrameReassembler::new();

        let stream = reassembler.stream();
        let jh = async_std::task::spawn(async move { stream.collect::<Vec<_>>().await });

        fragments.into_par_iter().for_each(|f| reassembler.add_frame(f));
        reassembler.close();

        jh.await.into_par_iter().for_each(|(seq_num, reassembled)| {
            assert_eq!(reassembled.as_ref(), packets[seq_num as usize].as_ref());
        });
    }
}

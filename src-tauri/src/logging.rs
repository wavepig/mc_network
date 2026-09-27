use std::collections::VecDeque;
use std::sync::Mutex;

pub struct LogBuffer {
    ring: Mutex<VecDeque<String>>,
    cap: usize,
}

impl LogBuffer {
    pub fn new(cap: usize) -> Self {
        Self { ring: Mutex::new(VecDeque::with_capacity(cap)), cap }
    }

    pub fn push(&self, line: impl Into<String>) {
        let mut ring = self.ring.lock().unwrap();
        if ring.len() == self.cap {
            ring.pop_front();
        }
        ring.push_back(line.into());
    }

    pub fn tail(&self, n: usize) -> Vec<String> {
        let ring = self.ring.lock().unwrap();
        ring.iter().rev().take(n).rev().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_keeps_last_cap_lines() {
        let log = LogBuffer::new(3);
        log.push("a"); log.push("b"); log.push("c"); log.push("d");
        assert_eq!(log.tail(10), vec!["b", "c", "d"]);
    }

    #[test]
    fn tail_respects_n() {
        let log = LogBuffer::new(5);
        log.push("a"); log.push("b"); log.push("c");
        assert_eq!(log.tail(2), vec!["b", "c"]);
    }

    #[test]
    fn empty_tail_is_empty() {
        assert!(LogBuffer::new(5).tail(3).is_empty());
    }
}

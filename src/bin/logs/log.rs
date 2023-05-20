use std::cmp;

pub struct Log {
    pub commited: i32,
    pub next: i32,
    pub msgs: Vec<i32>,
}

impl Log {
    pub fn push(&mut self, x: i32) -> i32 {
        self.next += 1;
        self.msgs.push(x);
        self.next - 1
    }

    pub fn poll(&self, offset: i32) -> Vec<(i32, i32)> {
        let skip = cmp::max(offset - self.commited - 1, 0);

        if skip >= self.msgs.len() as i32 {
            return Vec::new();
        }

        let start = cmp::max(offset, self.commited + 1);
        (start..).zip(self.msgs[skip as usize..].iter().copied()).collect()
    }

    pub fn commit(&mut self, offset: i32) {
        if offset <= self.commited {
            return;
        }

        let count = offset - self.commited;
        self.msgs.drain(..count as usize);
        self.commited = offset;
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            commited: 0,
            next: 1,
            msgs: Vec::new(),
        }
    }
}

mod tests {
    #[test]
    fn log_push() {
        let mut log = super::Log::default();
        assert_eq!(log.push(12), 1);
        assert_eq!(log.push(23), 2);
        assert_eq!(log.push(58), 3);
    }

    #[test]
    fn log_poll() {
        let mut log = super::Log::default();
        log.push(12);
        log.push(23);

        assert_eq!(log.poll(1), vec![(1, 12), (2, 23)]);
        assert_eq!(log.poll(2), vec![(2, 23)]);
        assert!(log.poll(3).is_empty());
        assert!(log.poll(4).is_empty());

        log.push(58);
        assert_eq!(log.poll(1), vec![(1, 12), (2, 23), (3, 58)]);
        assert_eq!(log.poll(2), vec![(2, 23), (3, 58)]);
        assert_eq!(log.poll(3), vec![(3, 58)]);
    }

    #[test]
    fn log_commit() {
        let mut log = super::Log::default();
        log.push(12);
        log.push(23);
        log.push(58);

        assert_eq!(log.poll(1), vec![(1, 12), (2, 23), (3, 58)]);
        log.commit(2);
        assert_eq!(log.poll(1), vec![(3, 58)]);

        assert_eq!(log.push(11), 4);
        assert_eq!(log.push(111), 5);

        assert_eq!(log.poll(1), vec![(3, 58), (4, 11), (5, 111)]);
        assert_eq!(log.poll(2), vec![(3, 58), (4, 11), (5, 111)]);
        assert_eq!(log.poll(4), vec![(4, 11), (5, 111)]);

        log.commit(4);

        assert_eq!(log.poll(1), vec![(5, 111)]);
        assert_eq!(log.poll(4), vec![(5, 111)]);
        assert_eq!(log.poll(5), vec![(5, 111)]);
    }
}

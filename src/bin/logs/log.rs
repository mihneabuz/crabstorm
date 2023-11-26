#[derive(Default, Debug, Clone)]
pub struct Log {
    inner: Vec<(i32, i32)>,
    commited: i32,
}

impl Log {
    pub fn push(&mut self, x: i32) -> i32 {
        let idx = self.inner.last().map(|l| l.0).unwrap_or(0) + 1;
        self.inner.push((idx, x));
        idx
    }

    pub fn poll(&self, offset: i32) -> Vec<(i32, i32)> {
        let index = match self.inner.binary_search_by_key(&offset, |&(i, _)| i) {
            Ok(found) => found,
            Err(insert) => insert,
        };

        self.inner[index..]
            .iter()
            .copied()
            .filter(|&(i, _)| i >= offset)
            .take(20)
            .collect()
    }

    pub fn commit(&mut self, offset: i32) {
        self.commited = offset;
    }

    pub fn commited(&self) -> i32 {
        self.commited
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
        assert_eq!(log.commited(), 2);
    }
}

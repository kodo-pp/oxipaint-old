use sdl2::pixels::Color;

pub struct History {
    diffs: Vec<Diff>,
    cursor: usize,
}

impl History {
    pub fn new() -> History {
        History {
            diffs: Vec::new(),
            cursor: 0,
        }
    }

    #[inline]
    fn consistency_check(&self) {
        assert!(self.cursor <= self.diffs.len());
    }

    pub fn undo(&mut self) -> Option<&Diff> {
        let diff = if self.cursor == 0 {
            None
        } else {
            self.cursor -= 1;
            Some(&self.diffs[self.cursor])
        };

        self.consistency_check();
        diff
    }

    pub fn redo(&mut self) -> Option<&Diff> {
        let diff = self.diffs.get(self.cursor);
        if diff.is_some() {
            self.cursor += 1;
        }
        self.consistency_check();
        diff
    }

    pub fn record(&mut self, diff: Diff) {
        self.consistency_check();
        self.diffs.reserve(self.cursor + 1);
        self.diffs.resize_with(self.cursor, || {
            panic!("It is a bug to increase the size of the history vector")
        });
        self.diffs.push(diff);
        self.cursor += 1;
        self.consistency_check();
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct SparsePixelDelta {
    pub index: usize,
    pub before: Color,
    pub after: Color,
}

pub enum Diff {
    Sparse(Vec<SparsePixelDelta>),
}

pub enum DiffDirection {
    Normal,
    Reverse,
}

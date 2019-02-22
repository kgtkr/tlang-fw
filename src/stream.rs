#[derive(Clone, Debug)]
pub struct Stream<T>(Vec<T>, usize);

impl<T: Clone> Stream<T> {
    pub fn peak(&self) -> Option<T> {
        self.0.get(self.1).cloned()
    }

    pub fn peak_index(&self, i: usize) -> Option<T> {
        self.0.get(self.1 + i).cloned()
    }
}

impl<T> Stream<T> {
    pub fn new(data: Vec<T>) -> Self {
        Stream(data, 0)
    }

    pub fn pos(&self) -> usize {
        self.1
    }

    pub fn set_pos(&mut self, pos: usize) -> Option<()> {
        if pos <= self.0.len() {
            self.1 = pos;
            Some(())
        } else {
            None
        }
    }

    pub fn add_pos(&mut self, x: usize) -> Option<()> {
        self.set_pos(self.pos() + x)
    }

    pub fn next(&mut self) -> Option<()> {
        self.add_pos(1)
    }

    pub fn eof(&self) -> bool {
        self.0.len() <= self.1
    }
}

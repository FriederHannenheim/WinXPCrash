
#[derive(Clone, Debug)]
pub struct RingBuffer {
    samples: Vec<f32>,
    size: usize,
    head: usize,
    pub freezing: bool,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        Self { 
            samples: vec![0.; crate::MAX_BUFFER_SIZE],
            size,
            head: 0,
            freezing: false,
        }
    }

    fn advance(&mut self) {
        self.head = (self.head + 1) % (self.size - 1);
    }


    pub fn next_item(&mut self, item: f32) -> f32 {
        self.advance();
        if self.freezing {
            self.samples[self.head]
        } else {
            self.samples[self.head] = item;
            item
        }
    }

    pub fn resize(&mut self, size: usize) {
        // Set all samples that are outside the new size to 0
        self.samples.iter_mut().skip(size).for_each(|s| *s = 0.);
        
        self.head = self.head.min(size - 1);
        self.size = size;
    }
}
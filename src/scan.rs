
use std::collections::HashMap;

#[derive(Debug,Default)]
pub struct Span {
    pub x: i64,
    pub len: i64,
    pub covers: Vec<u64>,
}
#[derive(Debug,Default)]
pub struct ScanlineU8 {
    last_x: i64,
    min_x: i64,
    pub spans: Vec<Span>,
    pub covers: HashMap<i64, u64>,
    pub y: i64,
}

const LAST_X: i64 = 0x7FFF_FFF0;
impl ScanlineU8 {
    pub fn new() -> Self {
        Self { last_x: LAST_X, min_x: 0, y: 0,
               spans: vec![], covers: HashMap::new() }
    }
    pub fn reset_spans(&mut self) {
        self.last_x = LAST_X;
        self.spans.clear();
        self.covers.clear();
    }
    pub fn reset(&mut self, min_x: i64, _max_x: i64) {
        self.last_x = LAST_X;
        self.min_x = min_x;
        self.spans = vec![];
        self.covers = HashMap::new()
    }
    pub fn finalize(&mut self, y: i64) {
        self.y = y;
    }
    pub fn num_spans(&self) -> usize {
        self.spans.len()
    }
    pub fn add_span(&mut self, x: i64, len: i64, cover: u64) {
        let x = x - self.min_x;
        self.covers.insert( x, cover );
        if x == self.last_x + 1 {
            let cur = self.spans.last_mut().unwrap();
            eprintln!("ADD_SPAN: Increasing length of span: {} {} x: {} {}", cur.len, cur.covers.len(), x+self.min_x, len);
            cur.len += len;
            cur.covers.extend(vec![cover; len as usize]);
            eprintln!("ADD_SPAN: Increasing length of span: {} {} x: {}", cur.len, cur.covers.len(), x+self.min_x);
        } else {
            eprintln!("ADD_SPAN: Adding span of length: {} at {}", len, x+self.min_x);
            let span = Span { x: x + self.min_x, len,
                              covers: vec![cover; len as usize] };
            self.spans.push(span);
        }
        self.last_x = x + len - 1;
    }
    pub fn add_cell(&mut self, x: i64, cover: u64) {
        let x = x - self.min_x;
        self.covers.insert( x, cover );
        if x == self.last_x + 1 {
            let cur = self.spans.last_mut().unwrap();
            cur.len += 1;
            cur.covers.push(cover);
        } else {
            //let cover = self.covers.get(&x).unwrap().clone();
            let span = Span { x: x + self.min_x, len: 1,
                              covers: vec![cover] };
            self.spans.push(span);
        }
        self.last_x = x;
    }
}

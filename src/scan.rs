//! Scanlines

//use std::collections::HashMap;

/// Contigious area of data
#[derive(Debug,Default)]
pub(crate) struct Span {
    /// Starting x position
    pub x: i64,
    /// Length of span
    pub len: i64,
    /// Cover values with len values
    pub covers: Vec<u64>,
}

/// Unpacked Scanline
///
/// Represents a single row of an image
#[derive(Debug,Default)]
pub(crate) struct ScanlineU8 {
    /// Last x value used
    ///
    /// Used as a state variable
    last_x: i64,
    /// Minimum x position
    ///
    /// This value can probably be removed
    min_x: i64,
    /// Collection of spans
    pub spans: Vec<Span>,
    // / Collection of covers
    // / Needed ?
    //covers: HashMap<i64, u64>,
    /// Current y value
    ///
    /// State variable
    pub y: i64,
}

const LAST_X: i64 = 0x7FFF_FFF0;

impl ScanlineU8 {
    /// Create a new empty scanline
    pub fn new() -> Self {
        Self { last_x: LAST_X, min_x: 0, y: 0,
               spans: Vec::with_capacity(256), } //covers: HashMap::new() }
    }
    /// Reset values and clear spans
    pub fn reset_spans(&mut self) {
        self.last_x = LAST_X;
        self.spans.clear();
        //self.covers.clear();
    }
    /// Reset values and clear spans, setting min value
    pub fn reset(&mut self, min_x: i64, _max_x: i64) {
        self.last_x = LAST_X;
        self.min_x = min_x;
        self.spans.clear();
        //self.covers = HashMap::new()
    }
    /// Set the current row (y) that is to be worked on
    pub fn finalize(&mut self, y: i64) {
        self.y = y;
    }
    /// Total number of spans
    pub fn num_spans(&self) -> usize {
        self.spans.len()
    }
    /// Add a span starting at x, with a length and cover value
    ///
    /// If the x value is 1 greater than the last value, the length of that
    /// span is increased and the cover value appended
    /// Otherwise, not a new span is created
    pub fn add_span(&mut self, x: i64, len: i64, cover: u64) {
        let x = x - self.min_x;
        //self.covers.insert( x, cover );
        if x == self.last_x + 1 {
            let cur = self.spans.last_mut().unwrap();
            cur.len += len;
            cur.covers.extend(vec![cover; len as usize]);
        } else {
            let span = Span { x: x + self.min_x, len,
                              covers: vec![cover; len as usize] };
            self.spans.push(span);
        }
        self.last_x = x + len - 1;
    }
    /// Add a single length span, cell, with a cover value
    ///
    /// If the cell is 1 beyond the last value, the length is increased and the
    /// cover is append, otherwise a new span is created
    pub fn add_cell(&mut self, x: i64, cover: u64) {
        let x = x - self.min_x;
        //self.covers.insert( x, cover );
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

use std::collections::VecDeque;
/// Application-level types for the root crate
#[derive(Debug)]
pub struct GcodeSendState {
    pub lines: VecDeque<String>,
    pub pending_bytes: usize,
    pub line_lengths: VecDeque<usize>,
    pub sent_lines: VecDeque<String>,
    pub total_sent: usize,
    pub total_lines: usize,
    pub start_time: Option<std::time::Instant>,
}

impl Default for GcodeSendState {
    fn default() -> Self {
        Self {
            lines: VecDeque::new(),
            pending_bytes: 0,
            line_lengths: VecDeque::new(),
            sent_lines: VecDeque::new(),
            total_sent: 0,
            total_lines: 0,
            start_time: None,
        }
    }
}

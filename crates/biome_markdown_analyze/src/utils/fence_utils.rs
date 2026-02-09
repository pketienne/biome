/// Information about a detected fenced code block opening.
pub struct FenceOpen {
    /// The fence character ('`' or '~').
    pub fence_char: char,
    /// Number of fence characters (>= 3).
    pub fence_count: usize,
    /// The info string (language tag) after the fence, trimmed.
    pub info_string: String,
    /// The 0-based line index of the opening fence.
    pub line_index: usize,
}

/// State tracker for fenced code block detection during line-by-line scanning.
///
/// Use this to determine whether a given line is inside a fenced code block.
///
/// # Example
///
/// ```ignore
/// let mut tracker = FenceTracker::new();
/// for (i, line) in text.lines().enumerate() {
///     tracker.process_line(i, line);
///     if tracker.is_inside_fence() {
///         // skip this line — it's inside a code block
///     }
/// }
/// ```
pub struct FenceTracker {
    /// If inside a fence, stores (fence_char, min_count).
    open_fence: Option<(char, usize)>,
}

impl FenceTracker {
    pub fn new() -> Self {
        Self { open_fence: None }
    }

    /// Returns `true` if we are currently inside a fenced code block
    /// (between opening and closing fences).
    pub fn is_inside_fence(&self) -> bool {
        self.open_fence.is_some()
    }

    /// Process a line and update the fence state.
    ///
    /// Returns `Some(FenceOpen)` if this line is an opening fence,
    /// `None` otherwise.
    pub fn process_line(&mut self, line_index: usize, line: &str) -> Option<FenceOpen> {
        let trimmed = line.trim_start();

        let fence_char = if trimmed.starts_with("```") {
            Some('`')
        } else if trimmed.starts_with("~~~") {
            Some('~')
        } else {
            None
        };

        let fence_char = fence_char?;
        let fence_count = trimmed.chars().take_while(|&c| c == fence_char).count();
        if fence_count < 3 {
            return None;
        }

        if let Some((open_char, open_count)) = self.open_fence {
            // We're inside a fence — check if this closes it
            let rest = trimmed[fence_count..].trim();
            if fence_char == open_char && fence_count >= open_count && rest.is_empty() {
                self.open_fence = None;
            }
            None
        } else {
            // Not inside a fence — this is an opening fence
            let info_string = trimmed[fence_count..].trim().to_string();
            self.open_fence = Some((fence_char, fence_count));
            Some(FenceOpen {
                fence_char,
                fence_count,
                info_string,
                line_index,
            })
        }
    }
}

impl Default for FenceTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_fence_detection() {
        let text = "before\n```js\ncode\n```\nafter";
        let mut tracker = FenceTracker::new();
        let lines: Vec<&str> = text.lines().collect();

        let r = tracker.process_line(0, lines[0]);
        assert!(r.is_none());
        assert!(!tracker.is_inside_fence());

        let r = tracker.process_line(1, lines[1]);
        assert!(r.is_some());
        let open = r.unwrap();
        assert_eq!(open.fence_char, '`');
        assert_eq!(open.fence_count, 3);
        assert_eq!(open.info_string, "js");
        assert!(tracker.is_inside_fence());

        let r = tracker.process_line(2, lines[2]);
        assert!(r.is_none());
        assert!(tracker.is_inside_fence());

        let r = tracker.process_line(3, lines[3]);
        assert!(r.is_none());
        assert!(!tracker.is_inside_fence());
    }

    #[test]
    fn tilde_fence() {
        let mut tracker = FenceTracker::new();
        let r = tracker.process_line(0, "~~~python");
        assert!(r.is_some());
        assert_eq!(r.unwrap().info_string, "python");
        assert!(tracker.is_inside_fence());

        tracker.process_line(1, "~~~");
        assert!(!tracker.is_inside_fence());
    }

    #[test]
    fn mismatched_fence_chars() {
        let mut tracker = FenceTracker::new();
        tracker.process_line(0, "```");
        assert!(tracker.is_inside_fence());

        // ~~~ does not close ``` fence
        tracker.process_line(1, "~~~");
        assert!(tracker.is_inside_fence());

        tracker.process_line(2, "```");
        assert!(!tracker.is_inside_fence());
    }

    #[test]
    fn fence_with_no_language() {
        let mut tracker = FenceTracker::new();
        let r = tracker.process_line(0, "```");
        assert!(r.is_some());
        assert_eq!(r.unwrap().info_string, "");
    }
}

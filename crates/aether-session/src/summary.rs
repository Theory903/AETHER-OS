//! Session summarizer — triggers when history exceeds threshold (PRD §16).

/// Policy for when to auto-summarize a session.
#[derive(Debug, Clone)]
pub struct SummaryPolicy {
    /// Summarize when history reaches this many messages.
    pub trigger_at_messages: usize,
    /// Retain this many messages after summarization.
    pub retain_after: usize,
}

impl Default for SummaryPolicy {
    fn default() -> Self {
        Self {
            trigger_at_messages: 40,
            retain_after: 10,
        }
    }
}

impl SummaryPolicy {
    /// Returns true when summarization should be triggered.
    pub fn should_summarize(&self, history_len: usize) -> bool {
        history_len >= self.trigger_at_messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trigger_fires_at_threshold() {
        let p = SummaryPolicy::default();
        assert!(!p.should_summarize(39));
        assert!(p.should_summarize(40));
    }

    #[test]
    fn test_custom_threshold() {
        let p = SummaryPolicy {
            trigger_at_messages: 10,
            retain_after: 3,
        };
        assert!(p.should_summarize(10));
        assert!(!p.should_summarize(9));
    }
}

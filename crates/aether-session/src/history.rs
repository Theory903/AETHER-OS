//! Conversation history management (PRD §16).

use aether_core::memory::{MemoryMessage, MessageRole};

/// Conversation history with truncation and summary support.
#[derive(Debug, Clone, Default)]
pub struct ConversationHistory {
    messages: Vec<MemoryMessage>,
}

impl ConversationHistory {
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a user message to the history.
    pub fn add_user(&mut self, content: impl Into<String>) {
        self.messages.push(MemoryMessage {
            role: MessageRole::User,
            content: content.into(),
        });
    }

    /// Append an assistant message to the history.
    pub fn add_assistant(&mut self, content: impl Into<String>) {
        self.messages.push(MemoryMessage {
            role: MessageRole::Assistant,
            content: content.into(),
        });
    }

    /// Append a tool result message.
    pub fn add_tool_result(&mut self, content: impl Into<String>) {
        self.messages.push(MemoryMessage {
            role: MessageRole::Tool,
            content: content.into(),
        });
    }

    /// Return all messages.
    pub fn messages(&self) -> &[MemoryMessage] {
        &self.messages
    }

    /// Total number of messages.
    pub fn len(&self) -> usize {
        self.messages.len()
    }

    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }

    /// Keep only the most recent `n` messages.
    pub fn truncate_to_last(&mut self, n: usize) {
        if self.messages.len() > n {
            let drop = self.messages.len() - n;
            self.messages.drain(0..drop);
        }
    }

    /// Replace all messages with a summary followed by recent context.
    pub fn replace_with_summary(&mut self, summary: &str, keep_last: usize) {
        let keep = self.messages.len().saturating_sub(keep_last);
        let recent: Vec<MemoryMessage> = self.messages.drain(keep..).collect();
        self.messages.clear();
        self.messages.push(MemoryMessage {
            role: MessageRole::System,
            content: format!("[Summary of earlier conversation]\n{summary}"),
        });
        self.messages.extend(recent);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn five_turn_history() -> ConversationHistory {
        let mut h = ConversationHistory::new();
        for i in 0..5 {
            h.add_user(format!("user message {i}"));
            h.add_assistant(format!("assistant response {i}"));
        }
        h
    }

    #[test]
    fn test_history_len() {
        let h = five_turn_history();
        assert_eq!(h.len(), 10);
    }

    #[test]
    fn test_truncate_drops_oldest() {
        let mut h = five_turn_history();
        h.truncate_to_last(4);
        assert_eq!(h.len(), 4);
        // Last message should be the last assistant response
        assert!(h.messages().last().unwrap().content.contains("4"));
    }

    #[test]
    fn test_truncate_no_op_when_short() {
        let mut h = ConversationHistory::new();
        h.add_user("hi");
        h.truncate_to_last(10);
        assert_eq!(h.len(), 1);
    }

    #[test]
    fn test_replace_with_summary_inserts_system_message() {
        let mut h = five_turn_history();
        h.replace_with_summary("They discussed greetings.", 2);
        // Should have: 1 system + 2 recent = 3
        assert_eq!(h.len(), 3);
        assert_eq!(h.messages()[0].role, MessageRole::System);
        assert!(h.messages()[0].content.contains("Summary"));
    }
}

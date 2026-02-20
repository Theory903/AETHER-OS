//! `aether-session` — Conversation session management (PRD §16).

pub mod history;
pub mod keys;
pub mod manager;
pub mod summary;

pub use history::ConversationHistory;
pub use keys::SessionKey;
pub use manager::{Session, SessionManager};

//! Application-wide constants.

// Tauri Event Names
pub mod events {
    // 3-section progressive events (words)
    pub const WORD_SECTION_1_HEADER: &str = "word-section-1-header";
    pub const WORD_SECTION_2_MEANINGS: &str = "word-section-2-meanings";
    pub const WORD_SECTION_3_RELATED: &str = "word-section-3-related";

    // 3-section progressive events (phrases)
    pub const PHRASE_SECTION_1_OVERVIEW: &str = "phrase-section-1-overview";
    pub const PHRASE_SECTION_2_CONTEXT: &str = "phrase-section-2-context";
    pub const PHRASE_SECTION_3_RELATED: &str = "phrase-section-3-related";
}

// API Endpoints
pub mod api {
    pub mod openai {
        pub const CHAT_COMPLETIONS: &str = "https://api.openai.com/v1/chat/completions";
        pub const MODELS: &str = "https://api.openai.com/v1/models";
    }

    pub mod deepseek {
        pub const CHAT_COMPLETIONS: &str = "https://api.deepseek.com/chat/completions";
        pub const MODELS: &str = "https://api.deepseek.com/models";
    }

    pub mod anthropic {
        pub const MESSAGES: &str = "https://api.anthropic.com/v1/messages";
        pub const MODELS: &str = "https://api.anthropic.com/v1/models";
        pub const VERSION: &str = "2023-06-01";
    }
}

// Provider Names
pub mod providers {
    pub const OPENAI: &str = "openai";
    pub const DEEPSEEK: &str = "deepseek";
    pub const ANTHROPIC: &str = "anthropic";
    pub const GEMINI: &str = "gemini";
    pub const OLLAMA: &str = "ollama";
}

pub mod explore_features {
    pub const FORMALITY: &str = "formality";
    pub const DOMAINS: &str = "domains";
    pub const USAGE: &str = "usage";
    pub const PRACTICE: &str = "practice";
    pub const MISTAKES: &str = "mistakes";
}

// HTTP Headers
pub mod headers {
    pub const X_API_KEY: &str = "x-api-key";
    pub const ANTHROPIC_VERSION: &str = "anthropic-version";
    pub const CONTENT_TYPE: &str = "content-type";
}

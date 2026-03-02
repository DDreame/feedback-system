-- Messages: individual chat messages within a conversation
CREATE TABLE messages (
    id              UUID         NOT NULL PRIMARY KEY,
    conversation_id UUID         NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_type     VARCHAR(20)  NOT NULL,
    sender_id       UUID,
    message_type    VARCHAR(20)  NOT NULL DEFAULT 'text',
    content         TEXT         NOT NULL,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_messages_sender_type CHECK (sender_type IN ('end_user', 'developer', 'system', 'ai')),
    CONSTRAINT ck_messages_message_type CHECK (message_type IN ('text', 'image', 'file'))
);

CREATE INDEX idx_messages_conversation_created ON messages(conversation_id, created_at);

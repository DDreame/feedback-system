-- End users: individuals who interact with a project via the SDK
CREATE TABLE end_users (
    id          UUID         NOT NULL PRIMARY KEY,
    project_id  UUID         NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    device_id   VARCHAR(255) NOT NULL,
    name        VARCHAR(255),
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    CONSTRAINT uq_end_users_project_device UNIQUE (project_id, device_id)
);

CREATE INDEX idx_end_users_project_id ON end_users(project_id);

-- Conversations: each support session between an end user and the developer team
CREATE TABLE conversations (
    id          UUID        NOT NULL PRIMARY KEY,
    project_id  UUID        NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    end_user_id UUID        NOT NULL REFERENCES end_users(id) ON DELETE CASCADE,
    status      VARCHAR(20) NOT NULL DEFAULT 'open',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT ck_conversations_status CHECK (status IN ('open', 'closed'))
);

CREATE INDEX idx_conversations_project_id ON conversations(project_id);
CREATE INDEX idx_conversations_end_user_id ON conversations(end_user_id);
CREATE INDEX idx_conversations_status ON conversations(status);

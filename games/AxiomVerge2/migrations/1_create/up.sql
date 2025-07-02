
CREATE TABLE axiom_verge2_states (
    raw_state BLOB NOT NULL, -- serialized Context
    progress INTEGER UNSIGNED NOT NULL DEFAULT 0, -- usize
    elapsed INTEGER UNSIGNED NOT NULL DEFAULT 0, -- u32
    time_since_visit INTEGER UNSIGNED NOT NULL DEFAULT 0, -- u32
    estimated_remaining INTEGER UNSIGNED NOT NULL DEFAULT 0,  -- u32
    processed BOOLEAN NOT NULL DEFAULT false,
    queued BOOLEAN NOT NULL DEFAULT false,
    won BOOLEAN NOT NULL DEFAULT false,
    hist BLOB,  -- serialized Vec<History>>
    prev BLOB,  -- serialized Context (which should be left serialized for prev lookup)
    PRIMARY KEY(raw_state(256)),
    INDEX(progress, won, processed, time_since_visit, (elapsed + estimated_remaining))
)
DATA DIRECTORY = "/mnt/e/.mysql";

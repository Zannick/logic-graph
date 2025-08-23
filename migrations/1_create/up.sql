
CREATE TABLE db_states (
    raw_state BLOB NOT NULL, -- serialized Context
    progress INTEGER UNSIGNED NOT NULL DEFAULT 0, -- usize
    elapsed INTEGER UNSIGNED NOT NULL DEFAULT 0, -- u32
    time_since_visit INTEGER UNSIGNED NOT NULL DEFAULT 0, -- u32
    estimated_remaining INTEGER UNSIGNED NOT NULL DEFAULT 0,  -- u32
    step_time INTEGER UNSIGNED NOT NULL DEFAULT 0,  -- u32
    processed BOOLEAN NOT NULL DEFAULT false,
    queued BOOLEAN NOT NULL DEFAULT false,
    won BOOLEAN NOT NULL DEFAULT false,
    hist TINYBLOB,  -- serialized History
    prev BLOB,  -- serialized Context (which should be left serialized for prev lookup)
    PRIMARY KEY(raw_state(256)),
    INDEX(progress),
    INDEX(processed),
    INDEX(queued),
    INDEX(won),
    INDEX(elapsed),
    INDEX((elapsed + estimated_remaining)),
    INDEX(time_since_visit),
    INDEX(prev(256))
)
DATA DIRECTORY = "/mnt/e/.mysql";  -- should be locally set by the user

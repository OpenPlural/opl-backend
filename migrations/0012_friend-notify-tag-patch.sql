ALTER TABLE Friend
    ADD COLUMN NotifyWithTag
        BOOLEAN NOT NULL
        DEFAULT FALSE
        AFTER NotifyMe;
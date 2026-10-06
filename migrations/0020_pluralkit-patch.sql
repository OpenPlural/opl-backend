ALTER TABLE Member
    ADD COLUMN PkId
        VARCHAR(6)
        DEFAULT NULL
        AFTER UserId;

ALTER TABLE User
    ADD COLUMN PkToken
        VARCHAR(64)
        DEFAULT NULL
        AFTER Email;

ALTER TABLE User
    ADD COLUMN PkDisplayName
        VARCHAR(255)
        DEFAULT NULL
        AFTER PkToken;
ALTER TABLE User
    ADD COLUMN AccountDisabled
        BOOLEAN NOT NULL
        DEFAULT FALSE
        AFTER PasswordResetTokenExpires;
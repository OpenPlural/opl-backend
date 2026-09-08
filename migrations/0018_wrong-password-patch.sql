ALTER TABLE User
    ADD COLUMN WrongPasswordEntries
        TINYINT UNSIGNED NOT NULL
        DEFAULT 0
        AFTER PasswordResetTokenExpires;
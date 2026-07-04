ALTER TABLE User
    ADD COLUMN PasswordResetToken
        VARCHAR(255)
        DEFAULT NULL
        UNIQUE
        AFTER Password;

ALTER TABLE User
    ADD COLUMN PasswordResetTokenExpires
        TIMESTAMP
        DEFAULT NULL
        AFTER PasswordResetToken;
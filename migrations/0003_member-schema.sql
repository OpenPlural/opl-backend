CREATE TABLE Member
(
    ID          BIGINT             NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId      INTEGER            NOT NULL,
    Name        VARCHAR(255)       NOT NULL,
    Pronouns    VARCHAR(255)                DEFAULT NULL,
    AvatarUrl   VARCHAR(255)                DEFAULT NULL,
    Description TEXT                        DEFAULT NULL,
    Color       MEDIUMINT UNSIGNED NOT NULL DEFAULT 16777215,
    CreatedAt   TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP(),
    UpdatedAt   TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP() ON UPDATE CURRENT_TIMESTAMP(),
    Archived    BOOLEAN            NOT NULL DEFAULT FALSE,
    Custom      BOOLEAN            NOT NULL DEFAULT FALSE,
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE
);

CREATE TRIGGER trig_delete_member
    BEFORE DELETE
    ON Member
    FOR EACH ROW
BEGIN
    INSERT INTO Deletion (ResourceId, ResourceType, UserId)
    VALUES (OLD.ID, 1, OLD.UserId);
END;
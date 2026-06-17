CREATE TABLE Folder
(
    ID          BIGINT             NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId      INTEGER            NOT NULL,
    ParentId    BIGINT                      DEFAULT NULL,
    Name        VARCHAR(255)       NOT NULL,
    Description TEXT                        DEFAULT NULL,
    Emoji       VARCHAR(30)                  DEFAULT NULL,
    Color       MEDIUMINT UNSIGNED NOT NULL DEFAULT 16777215,
    CreatedAt   TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP(),
    UpdatedAt   TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP() ON UPDATE CURRENT_TIMESTAMP(),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (ParentId) REFERENCES Folder (ID) ON DELETE CASCADE
);

CREATE TABLE MemberFolder
(
    UserId   INTEGER NOT NULL,
    MemberId BIGINT  NOT NULL,
    FolderId BIGINT  NOT NULL,
    PRIMARY KEY (UserId, MemberId, FolderId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (FolderId) REFERENCES Folder (ID) ON DELETE CASCADE,
    FOREIGN KEY (MemberId) REFERENCES Member (ID) ON DELETE CASCADE
);

CREATE TRIGGER trig_delete_folder
    BEFORE DELETE
    ON Folder
    FOR EACH ROW
BEGIN
    INSERT INTO Deletion (ResourceId, ResourceType, UserId)
    VALUES (OLD.ID, 0, OLD.UserId);
END;

CREATE TRIGGER trig_update_member_folder
    BEFORE UPDATE
    ON MemberFolder
    FOR EACH ROW
BEGIN
    UPDATE Member SET UpdatedAt = CURRENT_TIMESTAMP() WHERE ID = NEW.MemberId;
END;
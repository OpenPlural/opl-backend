CREATE TABLE Poll
(
    ID            BIGINT       NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId        INTEGER      NOT NULL,
    Name          VARCHAR(255) NOT NULL,
    Description   TEXT                  DEFAULT NULL,
    CustomOptions TEXT                  DEFAULT NULL,
    AllowAbstain  BOOLEAN      NOT NULL,
    AllowVeto     BOOLEAN      NOT NULL,
    OpenUntil     TIMESTAMP    NOT NULL DEFAULT TIMESTAMPADD(DAY, 7, CURRENT_TIMESTAMP()),
    UpdatedAt     TIMESTAMP    NOT NULL DEFAULT CURRENT_TIMESTAMP() ON UPDATE CURRENT_TIMESTAMP(),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE
);

CREATE TABLE PollAnswer
(
    ID        BIGINT           NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId    INTEGER          NOT NULL,
    PollId    BIGINT           NOT NULL,
    MemberId  BIGINT           NOT NULL,
    Answer    TINYINT UNSIGNED NOT NULL,
    Comment   VARCHAR(255)              DEFAULT NULL,
    UpdatedAt TIMESTAMP        NOT NULL DEFAULT CURRENT_TIMESTAMP() ON UPDATE CURRENT_TIMESTAMP(),
    UNIQUE (PollId, MemberId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (PollId) REFERENCES Poll (ID) ON DELETE CASCADE,
    FOREIGN KEY (MemberId) REFERENCES Member (ID) ON DELETE CASCADE
);

CREATE TRIGGER trig_delete_poll
    BEFORE DELETE
    ON Poll
    FOR EACH ROW
BEGIN
    INSERT INTO Deletion (ResourceId, ResourceType, UserId)
    VALUES (OLD.ID, 4, OLD.UserId);
END;

CREATE TRIGGER trig_delete_poll_answer
    BEFORE DELETE
    ON PollAnswer
    FOR EACH ROW
BEGIN
    INSERT INTO Deletion (ResourceId, ResourceType, UserId)
    VALUES (OLD.ID, 5, OLD.UserId);
END;
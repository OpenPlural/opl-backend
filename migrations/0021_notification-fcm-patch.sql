RENAME TABLE Notification TO WebPushSubscription;
ALTER TABLE WebPushSubscription
    DROP KEY UserId,
    ADD CONSTRAINT UserId UNIQUE (UserId, SessionId);

CREATE TABLE FCMSubscription
(
    ID             BIGINT       NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId         INTEGER      NOT NULL,
    SessionId      INTEGER      NOT NULL,
    InstallationId VARCHAR(255) NOT NULL,
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (SessionId) REFERENCES Session (ID) ON DELETE CASCADE,
    UNIQUE (UserId, SessionId)
);
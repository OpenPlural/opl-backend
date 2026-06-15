CREATE TABLE Notification
(
    ID        BIGINT       NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId    INTEGER      NOT NULL,
    SessionId INTEGER      NOT NULL,
    Endpoint  VARCHAR(255) NOT NULL,
    p256dh    VARCHAR(255) NOT NULL,
    auth      VARCHAR(255) NOT NULL,
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (SessionId) REFERENCES Session (ID) ON DELETE CASCADE,
    UNIQUE (UserId, SessionId, Endpoint, p256dh, auth)
);

CREATE TABLE LastNotification
(
    FrontingUserId  INTEGER      NOT NULL,
    ReceivingUserId INTEGER      NOT NULL,
    FrontText       VARCHAR(255) NOT NULL,
    PRIMARY KEY (FrontingUserId, ReceivingUserId),
    FOREIGN KEY (FrontingUserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (ReceivingUserId) REFERENCES User (ID) ON DELETE CASCADE
);
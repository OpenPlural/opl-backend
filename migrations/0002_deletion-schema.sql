CREATE TABLE Deletion
(
    ResourceId   BIGINT           NOT NULL,
    ResourceType TINYINT UNSIGNED NOT NULL,
    UserId       INTEGER          NOT NULL,
    ValidUntil   TIMESTAMP        NOT NULL DEFAULT TIMESTAMPADD(DAY, 8, CURRENT_TIMESTAMP()),
    PRIMARY KEY (ResourceId, ResourceType),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE
);
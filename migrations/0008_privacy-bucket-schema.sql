CREATE TABLE PrivacyBucket
(
    ID          BIGINT             NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId      INTEGER            NOT NULL,
    Sort        MEDIUMINT UNSIGNED NOT NULL,
    Name        VARCHAR(255)       NOT NULL,
    Description TEXT                        DEFAULT NULL,
    Emoji       VARCHAR(4)                  DEFAULT NULL,
    Color       MEDIUMINT UNSIGNED NOT NULL DEFAULT 16777215,
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE
);

CREATE TABLE PrivacyBucketFolder
(
    UserId   INTEGER NOT NULL,
    BucketId BIGINT  NOT NULL,
    FolderId BIGINT  NOT NULL,
    PRIMARY KEY (UserId, BucketId, FolderId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (BucketId) REFERENCES PrivacyBucket (ID) ON DELETE CASCADE,
    FOREIGN KEY (FolderId) REFERENCES Folder (ID) ON DELETE CASCADE
);

CREATE TABLE PrivacyBucketMember
(
    UserId   INTEGER NOT NULL,
    BucketId BIGINT  NOT NULL,
    MemberId BIGINT  NOT NULL,
    PRIMARY KEY (UserId, BucketId, MemberId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (BucketId) REFERENCES PrivacyBucket (ID) ON DELETE CASCADE,
    FOREIGN KEY (MemberId) REFERENCES Member (ID) ON DELETE CASCADE
);

CREATE TABLE PrivacyBucketCustomField
(
    UserId   INTEGER NOT NULL,
    BucketId BIGINT  NOT NULL,
    FieldId  BIGINT  NOT NULL,
    PRIMARY KEY (UserId, BucketId, FieldId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (BucketId) REFERENCES PrivacyBucket (ID) ON DELETE CASCADE,
    FOREIGN KEY (FieldId) REFERENCES CustomField (ID) ON DELETE CASCADE
);

CREATE TABLE PrivacyBucketFriend
(
    UserId   INTEGER NOT NULL,
    BucketId BIGINT  NOT NULL,
    FriendId INTEGER NOT NULL,
    PRIMARY KEY (UserId, BucketId, FriendId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (FriendId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (BucketId) REFERENCES PrivacyBucket (ID) ON DELETE CASCADE,
    FOREIGN KEY (UserId, FriendId) REFERENCES Friend (UserId, FriendId) ON DELETE CASCADE
);
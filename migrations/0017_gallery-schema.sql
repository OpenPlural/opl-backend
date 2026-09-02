CREATE TABLE PhotoAlbum
(
    ID          BIGINT             NOT NULL PRIMARY KEY AUTO_INCREMENT,
    UserId      INTEGER            NOT NULL,
    MemberId    BIGINT             NOT NULL,
    Sort        SMALLINT UNSIGNED  NOT NULL DEFAULT 0,
    Name        VARCHAR(255)       NOT NULL,
    Description TEXT                        DEFAULT NULL,
    PhotoUrls   TEXT                        DEFAULT NULL,
    UpdatedAt   TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP() ON UPDATE CURRENT_TIMESTAMP(),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (MemberId) REFERENCES Member (ID) ON DELETE CASCADE
);

CREATE TABLE PrivacyBucketPhotoAlbum
(
    UserId       INTEGER NOT NULL,
    BucketId     BIGINT  NOT NULL,
    PhotoAlbumId BIGINT  NOT NULL,
    PRIMARY KEY (UserId, BucketId, PhotoAlbumId),
    FOREIGN KEY (UserId) REFERENCES User (ID) ON DELETE CASCADE,
    FOREIGN KEY (BucketId) REFERENCES PrivacyBucket (ID) ON DELETE CASCADE,
    FOREIGN KEY (PhotoAlbumId) REFERENCES PhotoAlbum (ID) ON DELETE CASCADE
);

CREATE TRIGGER trig_delete_photo_album
    BEFORE DELETE
    ON PhotoAlbum
    FOR EACH ROW
BEGIN
    INSERT INTO Deletion (ResourceId, ResourceType, UserId)
    VALUES (OLD.ID, 6, OLD.UserId);
END;
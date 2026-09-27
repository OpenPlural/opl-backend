-- Schema Changes
ALTER TABLE Folder
    ADD COLUMN PrivacyPreview
        VARCHAR(255)
        DEFAULT NULL
        AFTER Color;

ALTER TABLE Member
    ADD COLUMN PrivacyPreview
        VARCHAR(255)
        DEFAULT NULL
        AFTER Color;

ALTER TABLE CustomField
    ADD COLUMN PrivacyPreview
        VARCHAR(255)
        DEFAULT NULL
        AFTER DataType;

ALTER TABLE PhotoAlbum
    ADD COLUMN PrivacyPreview
        VARCHAR(255)
        DEFAULT NULL
        AFTER PhotoUrls;

ALTER TABLE Friend
    ADD COLUMN PrivacyPreview
        VARCHAR(255)
        DEFAULT NULL
        AFTER NotifyWithTag;

-- Update Procedure
CREATE PROCEDURE update_privacy_preview(
    IN privacy_table VARCHAR(255),
    IN privacy_key VARCHAR(255),
    IN resource_table VARCHAR(255),
    IN resource_key VARCHAR(255),
    IN resource_id BIGINT,
    IN user_id INT
)
BEGIN
    PREPARE stmt FROM CONCAT(
        'UPDATE ', resource_table, ' rt ',
        'SET rt.PrivacyPreview = (',
            'SELECT GROUP_CONCAT(b.Emoji ORDER BY b.Sort SEPARATOR ", " ',
            'FROM ', privacy_table, ' l ',
            'JOIN PrivacyBucket b ON b.ID = l.BucketId ',
            'WHERE l.', privacy_key, ' = rt.', resource_key,
        ') ',
        'WHERE rt.', resource_key, ' = ? AND rt.UserId = ?'
    );
    EXECUTE stmt USING resource_id, user_id;
    DEALLOCATE PREPARE stmt;
END;

-- Triggers to call the update procedure
CREATE TRIGGER trig_update_folder_privacy_preview_on_insert
    AFTER INSERT
    ON PrivacyBucketFolder
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketFolder', 'FolderId', 'Folder', 'ID', NEW.FolderId, NEW.UserId);
END;

CREATE TRIGGER trig_update_folder_privacy_preview_on_delete
    AFTER DELETE
    ON PrivacyBucketFolder
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketFolder', 'FolderId', 'Folder', 'ID', OLD.FolderId, OLD.UserId);
END;

CREATE TRIGGER trig_update_member_privacy_preview_on_insert
    AFTER INSERT
    ON PrivacyBucketMember
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketMember', 'MemberId', 'Member', 'ID', NEW.MemberId, NEW.UserId);
END;

CREATE TRIGGER trig_update_member_privacy_preview_on_delete
    AFTER DELETE
    ON PrivacyBucketMember
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketMember', 'MemberId', 'Member', 'ID', OLD.MemberId, OLD.UserId);
END;

CREATE TRIGGER trig_update_custom_field_privacy_preview_on_insert
    AFTER INSERT
    ON PrivacyBucketCustomField
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketCustomField', 'FieldId', 'CustomField', 'ID', NEW.FieldId, NEW.UserId);
END;

CREATE TRIGGER trig_update_custom_field_privacy_preview_on_delete
    AFTER DELETE
    ON PrivacyBucketCustomField
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketCustomField', 'FieldId', 'CustomField', 'ID', OLD.FieldId, OLD.UserId);
END;

CREATE TRIGGER trig_update_photo_album_privacy_preview_on_insert
    AFTER INSERT
    ON PrivacyBucketPhotoAlbum
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketPhotoAlbum', 'PhotoAlbumId', 'PhotoAlbum', 'ID', NEW.PhotoAlbumId, NEW.UserId);
END;

CREATE TRIGGER trig_update_photo_album_privacy_preview_on_delete
    AFTER DELETE
    ON PrivacyBucketPhotoAlbum
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketPhotoAlbum', 'PhotoAlbumId', 'PhotoAlbum', 'ID', OLD.PhotoAlbumId, OLD.UserId);
END;

CREATE TRIGGER trig_update_friend_privacy_preview_on_insert
    AFTER INSERT
    ON PrivacyBucketFriend
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketFriend', 'FriendId', 'Friend', 'FriendId', NEW.FriendId, NEW.UserId);
END;

CREATE TRIGGER trig_update_friend_privacy_preview_on_delete
    AFTER DELETE
    ON PrivacyBucketFriend
    FOR EACH ROW
BEGIN
    CALL update_privacy_preview('PrivacyBucketFriend', 'FriendId', 'Friend', 'FriendId', OLD.FriendId, OLD.UserId);
END;

-- Migration of existing rows
UPDATE Folder f
SET f.PrivacyPreview = (
    SELECT GROUP_CONCAT(b.Emoji ORDER BY b.Sort SEPARATOR ', ')
    FROM PrivacyBucketFolder l
    JOIN PrivacyBucket b ON b.ID = l.BucketId
    WHERE l.FolderId = f.ID AND l.UserId = f.UserId
);

UPDATE Member m
SET m.PrivacyPreview = (
    SELECT GROUP_CONCAT(b.Emoji ORDER BY b.Sort SEPARATOR ', ')
    FROM PrivacyBucketMember l
    JOIN PrivacyBucket b ON b.ID = l.BucketId
    WHERE l.MemberId = m.ID AND l.UserId = m.UserId
);

UPDATE CustomField cf
SET cf.PrivacyPreview = (
    SELECT GROUP_CONCAT(b.Emoji ORDER BY b.Sort SEPARATOR ', ')
    FROM PrivacyBucketCustomField l
    JOIN PrivacyBucket b ON b.ID = l.BucketId
    WHERE l.FieldId = cf.ID AND l.UserId = cf.UserId
);

UPDATE PhotoAlbum pa
SET pa.PrivacyPreview = (
    SELECT GROUP_CONCAT(b.Emoji ORDER BY b.Sort SEPARATOR ', ')
    FROM PrivacyBucketPhotoAlbum l
    JOIN PrivacyBucket b ON b.ID = l.BucketId
    WHERE l.PhotoAlbumId = pa.ID AND l.UserId = pa.UserId
);

UPDATE Friend f
SET f.PrivacyPreview = (
    SELECT GROUP_CONCAT(b.Emoji ORDER BY b.Sort SEPARATOR ', ')
    FROM PrivacyBucketFriend l
    JOIN PrivacyBucket b ON b.ID = l.BucketId
    WHERE l.FriendId = f.FriendId AND l.UserId = f.UserId
);
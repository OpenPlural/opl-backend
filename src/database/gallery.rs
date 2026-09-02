use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::{query, Row};
use sqlx::mysql::MySqlRow;
use crate::database::{DatabaseExecutor, DatabasePool, DatabaseResult};
use crate::model::gallery::{PhotoAlbum, PhotoAlbumId, ViewedPhotoAlbum};
use crate::model::member::MemberId;
use crate::model::user::UserId;

pub async fn get_photo_album_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PhotoAlbumId>> {
    let ids = query("SELECT ID FROM PhotoAlbum WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_photo_albums(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PhotoAlbum>> {
    let albums = query("SELECT ID, UserId, MemberId, Sort, Name, Description, PhotoUrls, UpdatedAt FROM PhotoAlbum WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    let len = albums.len();
    let albums = albums.into_iter().try_fold(Vec::with_capacity(len), |mut acc, row| {
        match album(row) {
            Ok(poll) => {
                acc.push(poll);
                Ok(acc)
            }
            Err(err) => Err(err)
        }
    });
    albums.map_err(|err| anyhow!(err))
}

pub async fn get_updated_photo_albums(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<PhotoAlbum>> {
    let updated = query("SELECT ID, UserId, MemberId, Sort, Name, Description, PhotoUrls, UpdatedAt FROM PhotoAlbum WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    let len = updated.len();
    let updated = updated.into_iter().try_fold(Vec::with_capacity(len), |mut acc, row| {
        match album(row) {
            Ok(album) => {
                acc.push(album);
                Ok(acc)
            }
            Err(err) => Err(err)
        }
    });
    updated.map_err(|err| anyhow!(err))
}

pub async fn get_viewed_photo_albums_for_member(pool: &DatabasePool, user_id: UserId, member_id: MemberId, friend_viewer: Option<UserId>) -> DatabaseResult<Vec<ViewedPhotoAlbum>> {
    let res = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT ID, Sort, Name, Description, PhotoUrls
FROM PhotoAlbum pa
WHERE UserId = ? AND MemberId = ? AND EXISTS (
    SELECT 1 FROM PrivacyBucketPhotoAlbum ppa
             INNER JOIN PrivacyBucketFriend pf
             ON pf.BucketId = ppa.BucketId AND pf.UserId = ppa.UserId
             WHERE ppa.PhotoAlbumId = pa.ID AND pf.FriendId = ?
)
"#)
            .bind(user_id)
            .bind(member_id)
            .bind(friend_viewer)
            .fetch_all(pool.as_ref())
            .await?
    } else {
        query("SELECT ID, Sort, Name, Description, PhotoUrls FROM PhotoAlbum WHERE UserId = ? AND MemberId = ?")
            .bind(user_id)
            .bind(member_id)
            .fetch_all(pool.as_ref())
            .await?
    };

    let len = res.len();
    let res = res.into_iter().try_fold(Vec::with_capacity(len), |mut acc, row| {
        let photo_urls = row.get("PhotoUrls");
        let photo_urls = deserialize_photo_urls(&photo_urls);
        match photo_urls {
            Ok(photo_urls) => {
                let id = row.get("ID");
                let sort = row.get("Sort");
                let name = row.get("Name");
                let description = row.get("Description");
                acc.push(ViewedPhotoAlbum {
                    id,
                    sort,
                    name,
                    description,
                    photo_urls,
                });
                Ok(acc)
            }
            Err(err) => Err(err)
        }
    });
    res.map_err(|err| anyhow!(err))
}

pub async fn has_member_gallery(pool: &DatabasePool, member_id: MemberId, user_id: UserId, friend_viewer: Option<UserId>) -> DatabaseResult<bool> {
    let res = if let Some(friend_viewer) = friend_viewer {
        query(r#"
SELECT EXISTS(
    SELECT 1
    FROM PhotoAlbum pa
    WHERE UserId = ? AND MemberId = ? AND EXISTS (
        SELECT 1 FROM PrivacyBucketPhotoAlbum ppa
                 INNER JOIN PrivacyBucketFriend pf
                 ON pf.BucketId = ppa.BucketId AND pf.UserId = ppa.UserId
                 WHERE ppa.PhotoAlbumId = pa.ID AND pf.FriendId = ?
    )
)
"#)
            .bind(user_id)
            .bind(member_id)
            .bind(friend_viewer)
            .fetch_one(pool.as_ref())
            .await?
    } else {
        query("SELECT EXISTS(SELECT 1 FROM PhotoAlbum WHERE UserId = ? AND MemberId = ?)")
            .bind(user_id)
            .bind(member_id)
            .fetch_one(pool.as_ref())
            .await?
    };
    Ok(res.get(0))
}

pub async fn create_photo_album<'a, E: DatabaseExecutor<'a>>(executor: E, album: &PhotoAlbum) -> DatabaseResult<PhotoAlbumId> {
    let photo_urls = serialize_photo_urls(&album.photo_urls).map_err(|err| anyhow!(err))?;

    let id = query("INSERT INTO PhotoAlbum (UserId, MemberId, Sort, Name, Description, PhotoUrls) VALUES (?, ?, ?, ?, ?, ?) RETURNING ID")
        .bind(album.user_id)
        .bind(album.member_id)
        .bind(album.sort)
        .bind(&album.name)
        .bind(&album.description)
        .bind(photo_urls)
        .fetch_one(executor)
        .await?;

    Ok(id.get(0))
}

pub async fn delete_photo_album(pool: &DatabasePool, album_id: PhotoAlbumId, member_id: MemberId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM PhotoAlbum WHERE ID = ? AND UserId = ? AND MemberId = ?")
        .bind(album_id)
        .bind(user_id)
        .bind(member_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn delete_photo_album_by_id(pool: &DatabasePool, album_id: PhotoAlbumId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM PhotoAlbum WHERE ID = ? AND UserId = ?")
        .bind(album_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_photo_album(pool: &DatabasePool, album: &PhotoAlbum) -> DatabaseResult<()> {
    let photo_urls = serialize_photo_urls(&album.photo_urls).map_err(|err| anyhow!(err))?;

    query("UPDATE PhotoAlbum SET Sort = ?, Name = ?, Description = ?, PhotoUrls = ? WHERE ID = ? AND UserId = ? AND MemberId = ?")
        .bind(album.sort)
        .bind(&album.name)
        .bind(&album.description)
        .bind(photo_urls)
        .bind(album.id)
        .bind(album.user_id)
        .bind(album.member_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn get_photo_album_owner(pool: &DatabasePool, album_id: PhotoAlbumId) -> DatabaseResult<Option<UserId>> {
    let owner = query("SELECT UserId FROM PhotoAlbum WHERE ID = ?")
        .bind(album_id)
        .fetch_optional(pool.as_ref())
        .await?;

    Ok(owner.map(|row| row.get("UserId")))
}

fn album(row: MySqlRow) -> Result<PhotoAlbum, serde_json::Error> {
    let photo_urls = row.get("PhotoUrls");
    let photo_urls = deserialize_photo_urls(&photo_urls)?;

    let id = row.get("ID");
    let user_id = row.get("UserId");
    let member_id = row.get("MemberId");
    let sort = row.get("Sort");
    let name = row.get("Name");
    let description = row.get("Description");
    let updated_at = row.get("UpdatedAt");

    Ok(PhotoAlbum {
        id,
        user_id,
        member_id,
        sort,
        name,
        description,
        photo_urls,
        updated_at,
    })
}

fn serialize_photo_urls(photo_urls: &Option<Vec<String>>) -> Result<Option<String>, serde_json::Error> {
    if let Some(photo_urls) = photo_urls {
        Ok(Some(serde_json::to_string(photo_urls)?))
    } else {
        Ok(None)
    }
}

fn deserialize_photo_urls(photo_urls: &Option<String>) -> Result<Option<Vec<String>>, serde_json::Error> {
    if let Some(photo_urls) = photo_urls {
        Ok(Some(serde_json::from_str(&photo_urls)?))
    } else {
        Ok(None)
    }
}
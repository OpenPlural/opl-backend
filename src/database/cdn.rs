use anyhow::anyhow;
use sqlx::{query, Row};
use uuid::Uuid;
use crate::database::{DatabasePool, DatabaseResult};

const CDN_DIRECTORY: &str = "cdn";

pub async fn delete_unused_cdn_files(pool: &DatabasePool) -> DatabaseResult<()> {
    let used = query("SELECT AvatarUrl FROM Member WHERE AvatarUrl like ':cdn:%' UNION SELECT AvatarUrl FROM User WHERE AvatarUrl LIKE ':cdn:%' UNION SELECT AvatarUrl FROM PhotoAlbum pa JOIN JSON_TABLE(pa.PhotoUrls, '$[*]' COLUMNS(AvatarUrl TEXT COLLATE utf8mb4_unicode_ci PATH '$')) j WHERE AvatarUrl LIKE ':cdn:%'")
        .fetch_all(pool.as_ref())
        .await?;
    let used: Vec<Uuid> = used.into_iter()
        .map(|row| row.get(0))
        .filter_map(|cdn_data: String| {
            if let Some(cdn_data) = cdn_data.strip_prefix(":cdn:") {
                if let Some((cdn_uuid, _)) = cdn_data.split_once(':') {
                    return Uuid::parse_str(cdn_uuid).ok();
                }
            }
            None
        }).collect();

    let mut dir = tokio::fs::read_dir(CDN_DIRECTORY).await?;
    while let Some(entry) = dir.next_entry().await? {
        let uuid = entry.file_name().into_string().map_err(|name| anyhow!("{name:?} is not a valid string"))?;
        let uuid = Uuid::parse_str(&uuid)?;
        if used.contains(&uuid) {
            continue;
        }
        tokio::fs::remove_file(entry.path()).await?;
    }

    Ok(())
}

pub async fn store_image(data: impl AsRef<[u8]>) -> DatabaseResult<Uuid> {
    let mut id;
    let mut path;
    loop {
        id = Uuid::new_v4();
        path = format_path(&id);
        if !tokio::fs::try_exists(&path).await? {
            break;
        }
    }
    tokio::fs::write(path, data).await?;
    Ok(id)
}

pub async fn get_image(id: &Uuid) -> DatabaseResult<Vec<u8>> {
    let path = format_path(id);
    let data = tokio::fs::read(path).await?;
    Ok(data)
}

pub async fn has_image(id: &Uuid) -> DatabaseResult<bool> {
    let path = format_path(id);
    let exists = tokio::fs::try_exists(&path).await?;
    Ok(exists)
}

fn format_path(uuid: &Uuid) -> String {
    format!("./{}/{}", CDN_DIRECTORY, uuid.as_simple())
}
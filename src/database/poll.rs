use anyhow::anyhow;
use chrono::{DateTime, Utc};
use sqlx::mysql::MySqlRow;
use sqlx::{query, Row};
use crate::database::{DatabasePool, DatabaseResult};
use crate::model::poll::{Poll, PollAnswer, PollAnswerId, PollId};
use crate::model::user::UserId;

pub async fn get_poll_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PollId>> {
    let ids = query("SELECT ID FROM Poll WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_updated_polls(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<Poll>> {
    let updated = query("SELECT ID, UserId, Name, Description, CustomOptions, AllowAbstain, AllowVeto, OpenUntil, UpdatedAt FROM Poll WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    let len = updated.len();
    let updated = updated.into_iter().try_fold(Vec::with_capacity(len), |mut acc, row| {
        match poll(row) {
            Ok(poll) => {
                acc.push(poll);
                Ok(acc)
            }
            Err(err) => Err(err)
        }
    });
    updated.map_err(|err| anyhow!(err))
}

pub async fn get_polls(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<Poll>> {
    let polls = query("SELECT ID, UserId, Name, Description, CustomOptions, AllowAbstain, AllowVeto, OpenUntil, UpdatedAt FROM Poll WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    let len = polls.len();
    let polls = polls.into_iter().try_fold(Vec::with_capacity(len), |mut acc, row| {
        match poll(row) {
            Ok(poll) => {
                acc.push(poll);
                Ok(acc)
            }
            Err(err) => Err(err)
        }
    });
    polls.map_err(|err| anyhow!(err))
}

pub async fn get_poll_by_id(pool: &DatabasePool, poll_id: PollId, user_id: UserId) -> DatabaseResult<Option<Poll>> {
    let row = query("SELECT ID, UserId, Name, Description, CustomOptions, AllowAbstain, AllowVeto, OpenUntil, UpdatedAt FROM Poll WHERE ID = ? AND UserId = ?")
        .bind(poll_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;

    row.map(poll).transpose().map_err(|err| anyhow!(err))
}

pub async fn create_poll(pool: &DatabasePool, poll: &Poll) -> DatabaseResult<PollId> {
    let custom_options = poll_options(poll).map_err(|err| anyhow!(err))?;

    let id = query("INSERT INTO Poll (UserId, Name, Description, CustomOptions, AllowAbstain, AllowVeto, OpenUntil) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING ID")
        .bind(poll.user_id)
        .bind(&poll.name)
        .bind(&poll.description)
        .bind(custom_options)
        .bind(poll.allow_abstain)
        .bind(poll.allow_veto)
        .bind(&poll.open_until)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn delete_poll(pool: &DatabasePool, poll_id: PollId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM Poll WHERE ID = ? AND UserId = ?")
        .bind(poll_id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_poll(pool: &DatabasePool, poll: &Poll) -> DatabaseResult<()> {
    let custom_options = poll_options(poll).map_err(|err| anyhow!(err))?;

    query("UPDATE Poll SET Name = ?, Description = ?, CustomOptions = ?, AllowAbstain = ?, AllowVeto = ?, OpenUntil = ? WHERE ID = ? AND UserId = ?")
        .bind(&poll.name)
        .bind(&poll.description)
        .bind(custom_options)
        .bind(poll.allow_abstain)
        .bind(poll.allow_veto)
        .bind(&poll.open_until)
        .bind(poll.id)
        .bind(poll.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn is_custom_poll(pool: &DatabasePool, poll_id: PollId, user_id: UserId) -> DatabaseResult<Option<bool>> {
    let poll = query("SELECT CustomOptions FROM Poll WHERE ID = ? AND UserId = ?")
        .bind(poll_id)
        .bind(user_id)
        .fetch_optional(pool.as_ref())
        .await?;
    
    Ok(poll.map(|row| {
        let opts: Option<String> = row.get("CustomOptions");
        opts.is_some()
    }))
}

pub async fn get_poll_answer_ids(pool: &DatabasePool, user_id: UserId) -> DatabaseResult<Vec<PollId>> {
    let ids = query("SELECT ID FROM PollAnswer WHERE UserId = ?")
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(ids.into_iter().map(|row| row.get(0)).collect())
}

pub async fn get_updated_poll_answers(pool: &DatabasePool, user_id: UserId, newer_than: &DateTime<Utc>) -> DatabaseResult<Vec<PollAnswer>> {
    let updated = query("SELECT ID, UserId, PollId, MemberId, Answer, Comment, UpdatedAt FROM PollAnswer WHERE UserId = ? AND UpdatedAt > ?")
        .bind(user_id)
        .bind(newer_than)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(updated.into_iter().map(|row| {
        PollAnswer {
            id: row.get("ID"),
            user_id: row.get("UserId"),
            poll_id: row.get("PollId"),
            member_id: row.get("MemberId"),
            answer: row.get("Answer"),
            comment: row.get("Comment"),
            updated_at: row.get("UpdatedAt"),
        }
    }).collect())
}

pub async fn get_poll_answers(pool: &DatabasePool, poll_id: PollId, user_id: UserId) -> DatabaseResult<Vec<PollAnswer>> {
    let answers = query("SELECT ID, UserId, PollId, MemberId, Answer, Comment, UpdatedAt FROM PollAnswer WHERE PollId = ? AND UserId = ?")
        .bind(poll_id)
        .bind(user_id)
        .fetch_all(pool.as_ref())
        .await?;

    Ok(answers.into_iter().map(|row| {
        PollAnswer {
            id: row.get("ID"),
            user_id: row.get("UserId"),
            poll_id: row.get("PollId"),
            member_id: row.get("MemberId"),
            answer: row.get("Answer"),
            comment: row.get("Comment"),
            updated_at: row.get("UpdatedAt"),
        }
    }).collect())
}

pub async fn create_poll_answer(pool: &DatabasePool, answer: &PollAnswer) -> DatabaseResult<PollId> {
    let id = query("INSERT INTO PollAnswer (UserId, PollId, MemberId, Answer, Comment) VALUES (?, ?, ?, ?, ?) RETURNING ID")
        .bind(answer.user_id)
        .bind(answer.poll_id)
        .bind(answer.member_id)
        .bind(answer.answer)
        .bind(&answer.comment)
        .fetch_one(pool.as_ref())
        .await?;

    Ok(id.get(0))
}

pub async fn delete_poll_answer(pool: &DatabasePool, id: PollAnswerId, user_id: UserId) -> DatabaseResult<()> {
    query("DELETE FROM PollAnswer WHERE ID = ? AND UserId = ?")
        .bind(id)
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

pub async fn edit_poll_answer(pool: &DatabasePool, answer: &PollAnswer) -> DatabaseResult<()> {
    query("UPDATE PollAnswer SET Answer = ?, Comment = ? WHERE ID = ? AND UserId = ?")
        .bind(answer.answer)
        .bind(&answer.comment)
        .bind(answer.id)
        .bind(answer.user_id)
        .execute(pool.as_ref())
        .await?;

    Ok(())
}

fn poll(row: MySqlRow) -> Result<Poll, serde_json::Error> {
    let custom_options: Option<String> = row.get("CustomOptions");
    let custom_options: Option<Vec<String>> = if let Some(custom_options) = custom_options {
        Some(serde_json::from_str(&custom_options)?)
    } else {
        None
    };

    Ok(Poll {
        id: row.get("ID"),
        user_id: row.get("UserId"),
        name: row.get("Name"),
        description: row.get("Description"),
        custom_options,
        allow_abstain: row.get("AllowAbstain"),
        allow_veto: row.get("AllowVeto"),
        open_until: row.get("OpenUntil"),
        updated_at: row.get("UpdatedAt"),
    })
}

fn poll_options(poll: &Poll) -> Result<Option<String>, serde_json::Error> {
    if let Some(custom_options) = &poll.custom_options {
        Ok(Some(serde_json::to_string(custom_options)?))
    } else {
        Ok(None)
    }
}
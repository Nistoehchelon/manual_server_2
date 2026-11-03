use forum::Forum;
use futures::StreamExt;

use crate::{CanAccessThisForumInput, can_access_this_forum};

pub struct FilterForumListBasedOnWhatUserCanAccess {
    pub db_pool: sqlx::PgPool,
    pub user_id: String,
    pub forums: Vec<Forum>,
}
pub async fn filter_forum_list_based_on_what_user_can_access(
    data: FilterForumListBasedOnWhatUserCanAccess,
) -> Vec<Forum> {
    let FilterForumListBasedOnWhatUserCanAccess {
        db_pool,
        user_id,
        forums,
    } = data;
    futures::stream::iter(forums)
        .filter(|forum| {
            // clone the cheap stuff per future; `forum` is `&Forum`
            let db_pool = db_pool.clone();
            let user_id = user_id.clone();
            let forum_id = forum.id.clone();
            async move {
                can_access_this_forum(CanAccessThisForumInput {
                    db_pool,
                    user_id,
                    forum_id,
                })
                .await
                .unwrap_or(false) // fail closed
            }
        })
        .collect()
        .await
}

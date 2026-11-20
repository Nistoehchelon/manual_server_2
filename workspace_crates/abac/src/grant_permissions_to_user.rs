use sqlx::PgConnection;

use crate::permissions::Permission;

pub async fn add_permission_to_user(
    conn: &mut PgConnection,
    perm: Permission,
    user_id: String,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO access_attributes (name, subject, subject_id) VALUES ($1, $2, $3)",
        perm.to_string(),
        "user",
        user_id.clone()
    )
    .execute(conn)
    .await?;
    Ok(())
}
pub async fn add_admin_permissions_to_user(db_pool: sqlx::PgPool, user_id: String) -> () {
    let admin_perms = vec![
        Permission::SetCustomThreadIdWhenCreatingIt,
        Permission::ManageCustomSitePages,
        Permission::ManageCustomPageStylesheets,
        Permission::CanCreateForum,
        Permission::ViewAppExecutableVersion,
        Permission::CanEditSiteVariables,
        Permission::CanViewSiteVariables,
    ];
    for perm in admin_perms {
        let perm = perm.to_string();
        // I don't think I really care to handle the result since this is to be used just for startup the admin user for convenience
        let _ = sqlx::query!(
            "INSERT INTO access_attributes (name, subject, subject_id) VALUES ($1, $2, $3)",
            perm,
            "user",
            user_id.clone()
        )
        .execute(&db_pool)
        .await;
    }
}

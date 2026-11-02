pub async fn add_admin_permissions_to_user(db_pool: sqlx::PgPool, user_id: String) -> () {
    let admin_perms = vec![
        "set_custom_thread_id_when_creating_it",
        "manage_custom_site_pages",
        "create_forum",
        "view_app_executable_version",
        "manage_custom_page_stylesheets",
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

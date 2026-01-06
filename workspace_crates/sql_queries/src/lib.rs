use db_manager::DbParams;

/*
(params: DbParams<'_>) -> (&'_ str, DbParams<'_>) {
*/

pub fn get_user_by_user_id(params: DbParams<'_>) -> (&'_ str, DbParams<'_>) {

    ("SELECT primary_key, public_id, username, password_hash, password_salt, bio FROM public.users WHERE public_id = $1", params)
}

pub fn add_user(params: DbParams<'_>) -> (&'_ str, DbParams<'_>) {
    ("INSERT INTO public.users (public_id, username, password_hash, password_salt, bio) VALUES ($1, $2, $3, $4, $5)", params)
}
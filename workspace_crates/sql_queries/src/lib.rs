

pub fn get_user_by_user_id() -> String {

    format!("SELECT primary_key, id, username, password_hash, password_salt, bio FROM public.users WHERE id = $1")
}

pub fn add_user() -> String {
    format!("INSERT INTO public.users (id, username, password_hash, password_salt, bio) VALUES ($1, $2, $3, $4, $5)")
}
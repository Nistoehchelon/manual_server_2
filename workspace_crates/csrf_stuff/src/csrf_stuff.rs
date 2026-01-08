use std::fmt::{self, Display};

use db_manager::DbPool;
use error_structs::InternalAppError;


pub enum DesignatedPurpose {
    Normal,
}
impl Display for DesignatedPurpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Normal => "normal",

        };
        write!(f, "{}", s)
    }
}

pub async fn get_or_create_token(executor: &db_manager::DbPool, session_id: String, designated_purpose: DesignatedPurpose) -> Result<String, InternalAppError> {  
    let mut token_id: Option<String> = None;
    match db_manager::sql_queries::get_token_id_by_session_id_and_designated_purpose(executor, session_id.clone(), designated_purpose.to_string().clone()).await {
        Ok(t_opt) => {
            match t_opt {
                Some(t) => {
                    token_id = Some(t);
                }, 
                None => {
                }
                }
        }
        Err(e) => {
            dbg!(&e);
            return Err(InternalAppError::new("DB error".to_string()));
        }
    }
    if token_id.is_some() {
        return Ok(token_id.unwrap());
    }
    const LENGTH: usize = 33;
    let dp: String = designated_purpose.to_string();
    let token_id = id_generator::generate_random_alphanumeric_id(LENGTH);
    match db_manager::sql_queries::add_new_token(executor, token_id.clone(), dp.clone(), session_id.clone()).await {
       Ok(_) => {},
       Err(e) => {
        return Err(InternalAppError::new(e));
       }, 
    };

    return Ok(token_id);

}

pub async fn validate_token(executor: &DbPool, session_id: String, token_id: String) -> Result<bool, InternalAppError> {
    return db_manager::sql_queries::validate_token(executor, session_id, token_id).await.map_err(InternalAppError::new);
    
}
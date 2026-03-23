use password_hashing::{
    PasswordHasherConfigParams, hash_password, password_hashing::PasswordHasherConfig,
    verify_password,
};

#[test]
fn hashes_and_verifies_how_i_would_expect_it_to() {
    let initial_password = String::from("password123");
    let hash = hash_password(
        initial_password.clone(),
        PasswordHasherConfig::new(PasswordHasherConfigParams {
            m_cost: 16000,
            t_cost: 2,
            p_cost: 2,
        }),
    )
    .expect("I expected the hash to be generated successfully");
    dbg!(initial_password.clone());
    dbg!(hash.clone());

    // now imagine someone tries logging in with a valid password
    let valid_password = String::from("password123");
    dbg!(valid_password.clone());
    assert!(
        verify_password(valid_password.clone(), hash.clone(),)
            .expect("I expected the password verification to run successfully and return bool")
    );

    // now imagine someone did not enter the correct password
    let incorrect_password = String::from("tehee321");
    dbg!(incorrect_password.clone());
    assert!(
        !verify_password(incorrect_password, hash)
            .expect("I expected incorrect_password verification to return Ok(false)")
    );
}

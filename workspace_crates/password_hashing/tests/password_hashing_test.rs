use password_hashing::{HashAndSalt, hash_password_generate_salt, verify_password};

#[test]
fn hashes_and_verifies_how_i_would_expect_it_to() {
    let initial_password = String::from("password123");
    let HashAndSalt {
        hash: hash_of_initial_password,
        salt,
    } = hash_password_generate_salt(initial_password.clone());
    dbg!(initial_password.clone());
    dbg!(hash_of_initial_password.clone());
    dbg!(salt.clone());

    // now imagine someone tries logging in with a valid password
    let valid_password = String::from("password123");
    dbg!(valid_password.clone());
    assert!(verify_password(
        valid_password.clone(),
        hash_of_initial_password.clone(),
        salt.clone()
    ));

    // now imagine someone did not enter the correct password
    let incorrect_password = String::from("tehee321");
    dbg!(incorrect_password.clone());
    assert!(true != verify_password(incorrect_password, hash_of_initial_password, salt));
}

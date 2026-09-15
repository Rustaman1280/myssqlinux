use mysqldesk::database::DatabaseAuth;

#[test]
fn test_database_auth_test_empty_password() {
    // Check if test function runs and returns a boolean without panic
    let is_ok = DatabaseAuth::test_root_empty_password();
    println!("Database root empty password status: {}", is_ok);
}

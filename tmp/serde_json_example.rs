use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct User {
    username: String,
    password: String,
}

fn main() -> serde_json::Result<()> {
    let user_json = r#"{
        "username": "jckeep",
        "password": "123456"
    }"#;

    let user: User = serde_json::from_str(user_json)?;
    println!("{:#?}", user);
    Ok(())
}

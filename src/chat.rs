use crate::user::User;

pub struct Chat {
    users: Vec<User>,
}

impl Chat {
    pub fn new(users: Vec<User>) -> Self {
        Self { users }
    }
}

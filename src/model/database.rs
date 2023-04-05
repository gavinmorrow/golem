use super::{Message, User};

pub struct Database {
    users: Vec<User>,
    messages: Vec<Message>,
}

impl Database {
    pub fn new() -> Database {
        Database {
            users: Vec::new(),
            messages: Vec::new(),
        }
    }

    pub fn add_user(&mut self, user: User) -> Result<(), Error> {
        self.users.push(user);
        Ok(())
    }

    pub fn get_user(&self, id: super::user::Id) -> Option<&User> {
        self.users.iter().find(|user| user.id == id)
    }

    pub fn get_user_by_name(&self, name: &str) -> Option<&User> {
        self.users.iter().find(|user| user.name == name)
    }

    pub fn add_message(&mut self, message: Message) -> Result<(), Error> {
        self.messages.push(message);
        Ok(())
    }

    pub fn get_message(&self, id: super::message::Id) -> Option<&Message> {
        self.messages.iter().find(|message| message.id == id)
    }

    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }
}

#[derive(Debug)]
pub enum Error {
    DatabaseError,
}

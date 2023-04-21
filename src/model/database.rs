use super::{Message, Session, User};
use rusqlite::{Connection, Result};

pub struct Database {
    users: Vec<User>,
    messages: Vec<Message>,
	sessions: Vec<Session>,
}

impl Database {
    pub fn build() -> Result<Database> {
        let db = Database {
            sessions: Vec::new(),
        };
        db.init_db()?;
        Ok(db)
    }

    fn init_db(&self) -> Result<()> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE users (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL
            )",
            (), // empty list of parameters.
        )?;

        conn.execute(
            "CREATE TABLE messages (
                id      INTEGER PRIMARY KEY,
                author  INTEGER NOT NULL,
                parent  INTEGER,
                content TEXT NOT NULL,
                FOREIGN KEY(author) REFERENCES users(id),
                FOREIGN KEY(parent) REFERENCES messages(id)
            )",
            (), // empty list of parameters.
        )?;

        Ok(())
    }

    pub fn add_user(&mut self, user: User) -> Result<()> {
        self.users.push(user);
        Ok(())
    }

    pub fn get_user(&self, id: super::user::Id) -> Result<Option<&User>> {
        Ok(self.users.iter().find(|user| user.id == id))
    }

    pub fn get_user_by_name(&self, name: &str) -> Result<Option<&User>> {
        Ok(self.users.iter().find(|user| user.name == name))
    }

    pub fn add_message(&mut self, message: Message) -> Result<()> {
        self.messages.push(message);
        Ok(())
    }

    pub fn get_message(&self, id: super::message::Id) -> Option<&Message> {
        self.messages.iter().find(|message| message.id == id)
    }

    pub fn get_messages(&self) -> &Vec<Message> {
        &self.messages
    }

    pub fn add_session(&mut self, session: Session) -> Result<()> {
        self.sessions.push(session);
        Ok(())
    }

    pub fn get_session(&self, id: super::session::Id) -> Option<&Session> {
        self.sessions.iter().find(|session| session.id == id)
    }
}

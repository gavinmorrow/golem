use super::{Message, Session, User};
use rusqlite::{Connection, OptionalExtension, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn build() -> Result<Database> {
        let conn = Database::init_db()?;
        let db = Database { conn };
        Ok(db)
    }

    fn init_db() -> Result<Connection> {
        let conn = Connection::open_in_memory()?;

        conn.execute(
            "CREATE TABLE users (
                id   INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                password TEXT NOT NULL
            )",
            (),
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
            (),
        )?;

        conn.execute(
            "CREATE TABLE sessions (
                id      INTEGER PRIMARY KEY,
                user    INTEGER NOT NULL,
                FOREIGN KEY(user) REFERENCES users(id)
            )",
            (),
        )?;

        Ok(conn)
    }
}

impl Database {
    pub fn add_user(&self, user: User) -> Result<()> {
        self.conn.execute(
            "INSERT INTO users (id, name) VALUES (?1, ?2)",
            (user.id.id(), user.name),
        )?;
        Ok(())
    }

    pub fn get_user(&self, id: &super::user::Id) -> Result<Option<User>> {
        self.conn
            .query_row("SELECT * FROM users WHERE id=?1", (id.id(),), |row| {
                let id_row = row
                    .get::<usize, i64>(0)
                    .expect("id exists in query at column 0");
                Ok(User {
                    id: super::user::Id::try_from(id_row).expect("id from db is valid"),
                    name: row.get(1).expect("name exists in query at column 1"),
                    password: row.get(2).expect("password exists in query at column 2"),
                })
            })
            .optional()
    }

    pub fn get_user_by_name(&self, name: &str) -> Result<Option<User>> {
        self.conn
            .query_row("SELECT * FROM users WHERE name=?1", (name,), |row| {
                let id_row = row
                    .get::<usize, i64>(0)
                    .expect("id exists in query at column 0");
                Ok(User {
                    id: super::user::Id::try_from(id_row).expect("id from db is valid"),
                    name: row.get(1).expect("name exists in query at column 1"),
                    password: row.get(2).expect("password exists in query at column 2"),
                })
            })
            .optional()
    }
}

impl Database {
    pub fn add_session(&self, session: Session) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, user) VALUES (?1, ?2)",
            (session.id.id(), session.user_id.id()),
        )?;
        Ok(())
    }

    pub fn get_session(&self, id: &super::session::Id) -> Result<Option<Session>> {
        self.conn
            .query_row("SELECT * FROM users WHERE id=?1", (id.id(),), |row| {
                let session_id_row = row
                    .get::<usize, i64>(0)
                    .expect("id exists in query at column 0");
                let user_id_row = row
                    .get::<usize, i64>(1)
                    .expect("user_id exists in query at column 1");
                Ok(Session {
                    id: super::session::Id::try_from(session_id_row).expect("id from db is valid"),
                    user_id: super::user::Id::try_from(user_id_row).expect("id from db is valid"),
                })
            })
            .optional()
    }
}

impl Database {
    pub fn authenticate(&self, user_id: &super::user::Id, password: &String) -> Result<bool> {
        let user = self.get_user(user_id)?;
        Ok(user.map(|u| &u.password == password).unwrap_or(false))
    }
}

use super::{Message, Session, User, Snowflake};
use rusqlite::{types::FromSql, Connection, OptionalExtension, Result as SqlResult};

type Result<T> = SqlResult<Option<T>>;

pub struct Database {
    conn: Connection,
}

/// Build the database.
impl Database {
    pub fn build() -> SqlResult<Database> {
        let conn = Database::init_db()?;
        let db = Database { conn };
        Ok(db)
    }

    fn init_db() -> SqlResult<Connection> {
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
                token   INTEGER,
                user    INTEGER NOT NULL,
                FOREIGN KEY(user) REFERENCES users(id)
            )",
            (),
        )?;

        Ok(conn)
    }
}

/// User stuff
impl Database {
    pub fn add_user(&self, user: User) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO users (id, name, password) VALUES (?1, ?2, ?3)",
            (user.id.id(), user.name, user.password),
        )?;
        Ok(())
    }

    pub fn get_user(&self, id: &super::user::Id) -> Result<User> {
        self.conn
            .query_row("SELECT * FROM users WHERE id=?1", (id.id(),), |row| {
                Ok(User {
                    id: self.get_snowflake_column(row, 0),
                    name: self.get_column(row, 1),
                    password: self.get_column(row, 2),
                })
            })
            .optional()
    }

    pub fn get_user_by_name(&self, name: &str) -> Result<User> {
        self.conn
            .query_row("SELECT * FROM users WHERE name=?1", (name,), |row| {
                Ok(User {
                    id: self.get_snowflake_column(row, 0),
                    name: self.get_column(row, 1),
                    password: self.get_column(row, 2),
                })
            })
            .optional()
    }
}

/// Messages stuff
impl Database {
    pub fn get_recent_messages(&self) -> SqlResult<Vec<Message>> {
        let mut stmt = self
            .conn
            .prepare("SELECT * FROM messages ORDER BY id DESC LIMIT 100")?;
        let rows = stmt.query_map([], |row| {
            Ok(Message {
                id: self.get_snowflake_column(row, 0),
                author: self.get_snowflake_column(row, 1),
                parent: self.get_snowflake_column_optional(row, 2),
                content: self.get_column(row, 3),
            })
        })?;

        let mut messages = Vec::new();
        for message in rows {
            messages.push(message?);
        }

        Ok(messages)
    }

    pub fn add_message(&self, message: Message) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO messages (id, author, parent, content) VALUES (?1, ?2, ?3, ?4)",
            (
                message.id.id(),
                message.author.id(),
                message.parent.map(|id| id.id()),
                message.content,
            ),
        )?;
        Ok(())
    }
}

/// Session stuff
impl Database {
    pub fn add_session(&self, session: Session) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO sessions (id, token, user) VALUES (?1, ?2, ?3)",
            (session.id.id(), session.token, session.user_id.id()),
        )?;
        Ok(())
    }

    pub fn get_session(&self, id: &super::session::Id) -> Result<Session> {
        self.conn
            .query_row("SELECT * FROM sessions WHERE id=?1", (id.id(),), |row| {
                Ok(Session {
                    id: self.get_snowflake_column(row, 0),
                    token: self.get_column(row, 1),
                    user_id: self.get_snowflake_column(row, 2),
                })
            })
            .optional()
    }

    pub fn get_session_from_token(&self, token: &super::session::Token) -> Result<Session> {
        self.conn
            .query_row("SELECT * FROM sessions WHERE token=?1", (token,), |row| {
                Ok(Session {
                    id: self.get_snowflake_column(row, 0),
                    token: self.get_column(row, 1),
                    user_id: self.get_snowflake_column(row, 2),
                })
            })
            .optional()
    }

    pub fn delete_session(&self, id: &super::session::Id) -> SqlResult<()> {
        self.conn
            .execute("DELETE FROM sessions WHERE id=?1", (id.id(),))?;
        Ok(())
    }
}

/// Helper methods
impl Database {
    /// Get a row from a query result.
    /// It is just a wrapper around the [`rusqlite::Row::get()`] method,
    /// but it panics if the value does not exist.
    ///
    /// # Panics
    ///
    /// Panics if the value does not exist, or the type is incorrect.
    fn get_column<T: FromSql>(&self, row: &rusqlite::Row, index: usize) -> T {
        row.get::<usize, T>(index)
            .expect(format!("value exists at row {}", index).as_str())
    }

    /// Get a row from a query result, and parse it as a snowflake.
    /// It is just a wrapper around the [`Database::get_column()`] method, and the [`snowcloud::Snowflake::try_from()`] method.
    ///
    /// # Panics
    ///
    /// Panics if the value does not exist, or the value is not a valid snowflake.
    fn get_snowflake_column(&self, row: &rusqlite::Row, index: usize) -> super::Snowflake {
        super::Snowflake::try_from(self.get_column::<i64>(row, index))
            .expect("id from db is a valid snowflake")
    }

    /// Gets a row from a query result, and parse it as a snowflake.
    /// It is just a wrapper around the [`Database::get_column()`] method, and the [`snowcloud::Snowflake::try_from()`] method.
    fn get_snowflake_column_optional(
        &self,
        row: &rusqlite::Row,
        index: usize,
    ) -> Option<super::Snowflake> {
        let Ok(Some(id)) = row.get::<usize, Option<i64>>(index) else { return None };
        super::Snowflake::try_from(id).ok()
    }
}

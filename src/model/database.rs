use super::{Message, Room, Session, Snowflake, User};
use log::{debug, info, trace};
use rusqlite::{types::FromSql, Connection, OptionalExtension, Result as SqlResult, Row};
use serde::__private::de;

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
        let conn = Connection::open("./db.sqlite3")?;

        trace!("Opened database connection.");
        trace!("Initializing database...");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id   INT PRIMARY KEY,
                name TEXT NOT NULL,
                password TEXT NOT NULL
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS messages (
                id          INT PRIMARY KEY,
                author      INT NOT NULL,
                author_name TEXT NOT NULL,
                parent      INT NOT NULL,
                content     TEXT NOT NULL,
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS sessions (
                id      INT PRIMARY KEY,
                token   INT NOT NULL,
                user    INT NOT NULL,
                FOREIGN KEY(user) REFERENCES users(id)
            )",
            (),
        )?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS rooms (d
                id   INT PRIMARY KEY,
                name TEXT NOT NULL
            )",
            (),
        )?;

        info!("Finished initialized database");

        Ok(conn)
    }
}

/// User stuff
impl Database {
    pub fn add_user(&self, user: User) -> SqlResult<()> {
        debug!("Adding user {} to database", user.id.id());
        self.conn.execute(
            "INSERT INTO users (id, name, password) VALUES (?1, ?2, ?3)",
            (user.id.id(), user.name, user.password),
        )?;
        Ok(())
    }

    pub fn get_user(&self, id: &super::user::Id) -> Result<User> {
        debug!("Getting user {}", id.id());
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
        debug!("Getting user (name: {})", name);
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

    pub fn get_user_name(&self, id: &super::user::Id) -> Result<String> {
        debug!("Getting user name for user {}", id.id());
        self.conn
            .query_row("SELECT name FROM users WHERE id=?1", (id.id(),), |row| {
                Ok(self.get_column(row, 0))
            })
            .optional()
    }
}

/// Messages stuff
impl Database {
    // FIXME: Only selects top level messages
    pub fn get_recent_messages(&self) -> SqlResult<Vec<Message>> {
        self.get_some_messages(None, 100)
    }

    pub fn get_messages(&self) -> SqlResult<Vec<Message>> {
        trace!("Getting all messages");

        let mut stmt = self
            .conn
            .prepare("SELECT * FROM messages ORDER BY id DESC")?;
        let messages = stmt
            .query_map((), |row| self.map_message(row))?
            .collect::<SqlResult<Vec<_>>>();

        trace!("Got all messages");

        messages
    }

    /// Get the `amount` messages before the given message.
    /// If `before` is `None`, get the `amount` most recent messages.
    ///
    /// This will get at *least* `amount` (*top level*) messages, but may get more.
    ///
    /// This will only get top level messages.
    pub fn get_some_messages(
        &self,
        before: Option<Snowflake>,
        amount: u8,
    ) -> SqlResult<Vec<Message>> {
        // Get top level messages
        let mut stmt = self.conn.prepare(
            "SELECT * FROM messages WHERE parent IS NULL AND id <= ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let messages = stmt
            .query_map((before.map(|id| id.id()), amount), |row| {
                self.map_message(row)
            })?
            .collect::<SqlResult<Vec<_>>>();

        messages
    }

    pub fn get_children_of(
        &self,
        message: Option<&Snowflake>,
        depth: u8,
    ) -> SqlResult<Vec<Message>> {
        // Return an empty vector if the depth is 0
        if depth == 0 {
            return Ok(Vec::new());
        }

        // Start by getting only the direct children of the given message
        // Then, for each child, get the children of that child, and so on
        // until we reach the given depth

        let mut stmt = self
            .conn
            .prepare("SELECT * FROM messages WHERE parent=?1")?;

        let mut direct_children: Vec<Message> = stmt
            .query_map((message.map(|id| id.id()),), |row| self.map_message(row))?
            // Safe b/c the values are all set to be `Ok` above
            .map(|msg| msg.unwrap())
            .collect();

        let mut children: Vec<Message> = direct_children
            .iter()
            .map(|child| self.get_children_of(Some(&child.id), depth - 1))
            .flat_map(|result| match result {
                Ok(vec) => vec.into_iter().map(|item| Ok(item)).collect(),
                Err(er) => vec![Err(er)],
            })
            .collect::<SqlResult<_>>()?;

        let mut messages = Vec::new();
        messages.append(&mut direct_children);
        messages.append(&mut children);
        messages.sort_unstable();

        Ok(messages)
    }

    pub fn add_message(&self, message: &Message) -> SqlResult<()> {
        debug!("Adding message {} to database", message.id.id());

        self.conn.execute(
            "INSERT INTO messages (id, author, author_name, parent, content) VALUES (?1, ?2, ?3, ?4, ?5)",
            (
                message.id.id(),
                message.author.id(),
                message.author_name.as_str(),
                message.parent.id(),
                message.content.as_str(),
            ),
        )?;

        debug!("Added message {} to database", message.id.id());

        Ok(())
    }

    fn map_message(&self, row: &Row) -> SqlResult<Message> {
        trace!("Mapping db row to message");

        let id = self.get_snowflake_column(row, 0);
        let author = self.get_snowflake_column(row, 1);
        let author_name = self.get_column(row, 2);
        let parent = self.get_snowflake_column(row, 3);
        let content = self.get_column(row, 4);

        Ok(Message {
            id,
            author,
            author_name,
            parent,
            content,
        })
    }
}

/// Room stuff
impl Database {
    pub fn add_room(&self, room: &Room) -> SqlResult<()> {
        debug!("Adding room {} to database", room.id.id());

        self.conn.execute(
            "INSERT INTO rooms (id, name) VALUES (?1, ?2)",
            (room.id.id(), room.name.as_str()),
        )?;

        debug!("Added room {} to database", room.id);
        info!("Created room {}: {}", room.id, room.name);

        Ok(())
    }

    pub fn get_room(&self, id: crate::model::room::Id) -> Result<Room> {
        debug!("Getting room {} from database", id);

        self.conn
            .query_row("SELECT * FROM rooms WHERE id=?1", (id.id(),), |row| {
                Ok(Room {
                    id: self.get_snowflake_column(row, 0),
                    name: self.get_column(row, 1),
                })
            })
            .optional()
    }
}

/// Session stuff
impl Database {
    pub fn add_session(&self, session: Session) -> SqlResult<()> {
        debug!("Adding session: {:?}", session.id.id());
        self.conn.execute(
            "INSERT INTO sessions (id, token, user) VALUES (?1, ?2, ?3)",
            (session.id.id(), session.token, session.user_id.id()),
        )?;
        Ok(())
    }

    pub fn get_session_from_token(&self, token: &super::session::Token) -> Result<Session> {
        debug!("Getting session from token {}", token);
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
        debug!("Deleting session {}", id.id());
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

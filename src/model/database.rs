use super::{Session, User};
use rusqlite::{types::FromSql, Connection, OptionalExtension, Result};

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
                token   INTEGER PRIMARY KEY,
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
                Ok(User {
                    id: self.get_snowflake_column(row, 0),
                    name: self.get_column(row, 1),
                    password: self.get_column(row, 2),
                })
            })
            .optional()
    }

    pub fn get_user_by_name(&self, name: &str) -> Result<Option<User>> {
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

impl Database {
    pub fn add_session(&self, session: Session) -> Result<()> {
        self.conn.execute(
            "INSERT INTO sessions (token, user) VALUES (?1, ?2)",
            (session.token, session.user_id.id()),
        )?;
        Ok(())
    }

    pub fn get_session(&self, token: &super::session::Token) -> Result<Option<Session>> {
        self.conn
            .query_row("SELECT * FROM users WHERE token=?1", (token,), |row| {
                Ok(Session {
                    id: self.get_snowflake_column(row, 0),
                    token: self.get_column(row, 1),
                    user_id: self.get_snowflake_column(row, 2),
                })
            })
            .optional()
    }
}

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
}

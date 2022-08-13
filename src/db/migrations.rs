use rusqlite_migration::{Migrations, M};

pub fn migrations() -> Migrations<'static> {
    Migrations::new(vec![M::up(
        r#"
            CREATE TABLE IF NOT EXISTS secure_data
            (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                account TEXT NULL,
                user TEXT NULL,
                token TEXT NULL,
                password TEXT NULL,
                note TEXT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )
            "#,
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_test() {
        assert!(migrations().validate().is_ok());
    }
}

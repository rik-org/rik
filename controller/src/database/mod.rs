use crate::api::types::element::Element;

use dotenv::dotenv;
use rusqlite::{params, Connection, Result};
use std::sync::Arc;
use uuid::Uuid;

#[allow(dead_code)]
pub struct RikDataBase {
    name: String,
}

#[allow(dead_code)]
impl RikDataBase {
    pub fn new(name: String) -> Arc<RikDataBase> {
        Arc::new(RikDataBase { name })
    }

    pub fn init_tables(&self) -> Result<()> {
        let connection = self.open()?;
        // only work with sqlite for now
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS cluster (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                value           BLOB NOT NULL
            );
            CREATE INDEX IF NOT EXISTS cluster_name_index ON cluster (name);
            CREATE INDEX IF NOT EXISTS cluster_name_id_index ON cluster (name,id);",
        )?;
        Ok(())
    }

    pub fn drop_tables(&self) {}

    pub fn open(&self) -> Result<Connection> {
        dotenv().ok();
        let file_path =
            std::env::var("DATABASE_LOCATION").unwrap_or_else(|_| "/var/lib/rik/data/".to_string());
        std::fs::create_dir_all(&file_path).unwrap();

        let database_path = format!("{}{}.db", file_path, self.name);
        Connection::open(&database_path)
    }
}

pub struct RikRepository {}
impl RikRepository {
    pub fn insert(connection: &Connection, name: &str, value: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        connection
            .execute(
                "INSERT INTO cluster (id, name, value) VALUES (?1, ?2, ?3)",
                params![id, name, value],
            )
            .unwrap();
        Ok(id)
    }

    pub fn delete(connection: &Connection, id: &String) -> Result<()> {
        connection.execute("DELETE FROM cluster WHERE id = (?1)", params![id])?;
        Ok(())
    }

    pub fn find_one(connection: &Connection, id: &String, element_type: &str) -> Result<Element> {
        let mut stmt = connection.prepare(&format!(
            "SELECT id, name, value FROM cluster WHERE id = '{}' AND name LIKE '{}%'",
            id, element_type
        ))?;
        match stmt.query_row([], |row| {
            Ok(Element::new(row.get(0)?, row.get(1)?, row.get(2)?))
        }) {
            Ok(element) => Ok(element),
            Err(err) => Err(err),
        }
    }

    pub fn check_duplicate_name(connection: &Connection, name: &str) -> Result<Element> {
        let mut stmt = connection.prepare(&format!(
            "SELECT id, name, value FROM cluster WHERE name LIKE '{}%'",
            name
        ))?;
        match stmt.query_row([], |row| {
            Ok(Element::new(row.get(0)?, row.get(1)?, row.get(2)?))
        }) {
            Ok(element) => Ok(element),
            Err(err) => Err(err),
        }
    }

    // TODO: add pagination
    pub fn find_all(connection: &Connection, element_type: &str) -> Result<Vec<Element>> {
        let mut stmt = connection
            .prepare(&format!(
                "SELECT id, name, value FROM cluster WHERE name LIKE '{}%'",
                element_type
            ))
            .unwrap();
        let elements_iter = stmt
            .query_map([], |row| {
                Ok(Element::new(row.get(0)?, row.get(1)?, row.get(2)?))
            })
            .unwrap();

        let mut elements: Vec<Element> = Vec::new();
        for element in elements_iter {
            elements.push(element?);
        }
        Ok(elements)
    }

    pub fn update(connection: &Connection, id: &String, value: &String) -> Result<()> {
        connection.execute(
            "UPDATE cluster SET value=(?1) WHERE id = (?2)",
            params![value, id],
        )?;
        Ok(())
    }

    pub fn upsert(
        connection: &Connection,
        id: &String,
        name: &String,
        value: &String,
        element_type: &str,
    ) -> Result<String> {
        if RikRepository::find_one(connection, id, element_type).is_ok() {
            RikRepository::update(connection, id, value)?;
            Ok(id.to_string())
        } else {
            connection
                .execute(
                    "INSERT INTO cluster (id, name, value) VALUES (?1, ?2, ?3)",
                    params![id, name, value],
                )
                .unwrap();
            Ok(id.to_string())
        }
    }
}

#[cfg(test)]
mod test {
    use crate::database::{RikDataBase, RikRepository};
    use crate::tests::fixtures::db_connection;
    use rstest::rstest;
    use uuid::Uuid;

    #[rstest]
    fn test_insert_and_find_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();
        let name = "/workload/pods/default/test-workload";
        let value = "{\"data\": \"test\"}";
        let inserted_id = match RikRepository::insert(&connection, name, value) {
            Ok(v) => v,
            Err(_) => panic!("Test failed on database insert"),
        };

        let element = match RikRepository::find_one(&connection, &inserted_id, "/workload") {
            Ok(v) => v,
            Err(_) => panic!("Test failed can't find inserted value"),
        };
        assert_eq!(element.id, inserted_id);
        assert_eq!(element.name, name);
        assert_eq!(element.value, serde_json::json!({"data": "test"}));
    }

    #[rstest]
    fn test_insert_and_find_all_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();
        match RikRepository::insert(
            &connection,
            "/workload/pods/default/test-workload",
            "{\"data\": \"test\"}",
        ) {
            Ok(v) => v,
            Err(_) => panic!("Test failed on database insert"),
        };
        match RikRepository::insert(
            &connection,
            "/workload/pods/default/test-workload2",
            "{\"data\": \"test\"}",
        ) {
            Ok(v) => v,
            Err(_) => panic!("Test failed on database insert"),
        };

        let elements = match RikRepository::find_all(&connection, "/workload") {
            Ok(v) => v,
            Err(_) => panic!("Test failed can't find inserted value"),
        };
        assert_eq!(elements.len(), 2);
    }

    #[rstest]
    fn test_check_duplicate_name(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();
        let name = "/workload/pods/default/test-workload";
        let value = "{\"data\": \"test\"}";
        let inserted_id = match RikRepository::insert(&connection, name, value) {
            Ok(v) => v,
            Err(_) => panic!("Test failed on database insert"),
        };
        let duplicate = match RikRepository::check_duplicate_name(
            &connection,
            &format!("/workload/%/default/{}", "test-workload"),
        ) {
            Ok(v) => v,
            Err(_) => panic!("Test failed on check duplicate"),
        };
        assert_eq!(duplicate.id, inserted_id);
        assert_eq!(duplicate.name, name);
        assert_eq!(duplicate.value, serde_json::json!({"data": "test"}));
    }

    #[rstest]
    fn test_upsert_ok(db_connection: std::sync::Arc<RikDataBase>) {
        let connection = db_connection.open().unwrap();
        connection.execute("DELETE FROM cluster", []).unwrap();
        let id = Uuid::new_v4().to_string();
        let name = "/workload/pods/default/test-workload".to_string();
        let mut value = "{\"data\": \"test\"}".to_string();
        let inserted_id = match RikRepository::upsert(
            &connection,
            &id,
            &name,
            &value,
            &"/workload".to_string(),
        ) {
            Ok(v) => v,
            Err(e) => panic!("Test failed on first insert : {:?}", e),
        };
        let element = match RikRepository::find_one(&connection, &inserted_id, "/workload") {
            Ok(v) => v,
            Err(_) => panic!("Test failed can't find inserted value"),
        };
        assert_eq!(element.id, inserted_id);
        assert_eq!(element.name, name);
        assert_eq!(element.value, serde_json::json!({"data": "test"}));

        value = "{\"data\": \"test_updated\"}".to_string();
        match RikRepository::upsert(&connection, &id, &name, &value, &"/workload".to_string()) {
            Ok(v) => v,
            Err(e) => panic!("Test failed on first insert : {:?}", e),
        };
        let updated_element = match RikRepository::find_one(&connection, &inserted_id, "/workload")
        {
            Ok(v) => v,
            Err(_) => panic!("Test failed can't find inserted value"),
        };
        assert_eq!(updated_element.id, inserted_id);
        assert_eq!(updated_element.name, name);
        assert_eq!(
            updated_element.value,
            serde_json::json!({"data": "test_updated"})
        );
    }
}

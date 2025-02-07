use std::fs::File;
use std::path::Path;
use std::io::prelude::*;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex};
use serde::de::DeserializeOwned;
use serde::{Serialize, Deserialize};
use serde_json::Value;


#[derive(Serialize, Deserialize, Debug)]
struct TableData {
    next_id: u32,
    data: BTreeMap<u32, Value>
}

pub struct Db {
    file: Arc<Mutex<File>>,
    tables: HashMap<String, TableData>
}

type DynaResult<'a, T> = Result<T, Box<dyn std::error::Error + 'a>>;

impl Db {
    
    pub fn init(file_name: String) -> DynaResult<'static ,Self>{
        let path_exists = Path::new(&file_name).exists();

        let mut file = std::fs::OpenOptions::new()
            .write(true).read(true).create(true).open(file_name)?;


        let mut tables: HashMap<String, TableData> = HashMap::new();

        if path_exists {
            let mut contents = String::new();

            file.read_to_string(&mut contents).expect("Unable to read file!");

            if contents.len() > 0 {
                tables = serde_json::from_str(&contents).expect("Unreadable JSON!");
            }
        }

        let file_ref = Arc::new(Mutex::new(file));


        Ok(Self {
            file: file_ref,
            tables
        })
    }

    fn write(&mut self, data: String) -> DynaResult<'_, ()>{
        let mut file = self.file.lock()?;
        file.set_len(0)?;
        file.rewind()?;
        file.write_all(data.as_bytes())?;

        Ok(())
    }

    fn flush(&mut self) -> DynaResult<'_, ()> {
        let contents = serde_json::to_string(&self.tables).expect("Failed to flush to file!");

        return self.write(contents)
    }

    pub fn add_table(&mut self, table_name: String, is_recreate: bool) -> DynaResult<'_, ()> {
        if !is_recreate && self.tables.contains_key(&table_name) {
            println!("Table already exists!");
            return Ok(())
        }

        self.tables.insert(
            table_name, 
            TableData{ 
                next_id: 1,
                data: BTreeMap::new()
             });
        self.flush()?;

        Ok(())
    }

    pub fn find_all<T>(&self, table_name: String) -> Option<Vec<T>> 
        where T: DeserializeOwned
    {
        if let Some(table) = self.tables.get(&table_name) {
            return Some(
                table
                    .data
                    .values()
                    .cloned()
                    .map(|x| serde_json::from_value::<T>(x).unwrap())
                    .collect()
            );
        }

        return None
    }

    pub fn find_by_id<T>(&self, table_name: String, id: u32) -> Option<T> 
        where T: DeserializeOwned
    {
        if let Some(table) = self.tables.get(&table_name) {
            return table
                .data
                .get(&id)
                .cloned()
                .map(|x| serde_json::from_value::<T>(x).unwrap());
        }

        return None
    }

    pub fn get_increment_last_id(&mut self, table_name: String) -> DynaResult<'_, Option<u32>> {
        if let Some(table) = self.tables.get_mut(&table_name) {
            let id = table.next_id;
            table.next_id = id + 1;
            self.flush()?;
            return Ok(Some(id));
        }

        println!("Table does not exist! Cannot get next id.");
        return Ok(None)
    }

    pub fn insert_or_update<T>(&mut self, table_name: String, id: u32, data: T) -> DynaResult<'_, Option<T>> 
        where T: Serialize + Clone
    {
        if let Some(table) = self.tables.get_mut(&table_name) {
            table.data.insert(id, serde_json::to_value(data.clone())?);
            self.flush()?;
            return Ok(Some(data))
        }

        return Ok(None)
    }

    pub fn delete_by_id(&mut self, table_name: String, id: u32) -> DynaResult<'_, Option<Value>> {
        if let Some(table) = self.tables.get_mut(&table_name) {
            let data = table.data.remove(&id);
            self.flush()?;
            return Ok(data)
        }

        return Ok(None)
    }

    pub fn delete_all(&mut self, table_name: String) -> DynaResult<'_, bool> {
        if let Some(table) = self.tables.get_mut(&table_name) {
            table.data.clear();
            self.flush()?;
            return Ok(true)
        }

        return Ok(false)
    }
 }

 #[cfg(test)]
 mod tests {
    use serde_json::json;

    use crate::test::run_with_file_create_teardown;

    use super::*;

    static FILE_NAME: &str = "./data.json";

    #[test]
    fn test_init() {
        run_with_file_create_teardown(|| {
            let db = Db::init(String::from(FILE_NAME));

            assert!(db.is_ok())
        });
    }

    #[test]
    fn test_add_table() {
        run_with_file_create_teardown(|| {
            let mut db = Db::init(String::from(FILE_NAME)).unwrap();
            let table_name = String::from("sample");
            db.add_table(table_name.clone(), true).unwrap();

            let result = db.find_all::<Value>(table_name);

            assert_eq!(result.is_some(), true);
        });
    }

    #[test]
    fn test_insert() {
        run_with_file_create_teardown(|| {
            let mut db = Db::init(String::from(FILE_NAME)).unwrap();
            let table_name = String::from("sample");
            db.add_table(table_name.clone(), true).unwrap();

            let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
            let to_insert: Value = json!({"id": id, "value": "sample"});
            db.insert_or_update::<Value>(table_name.clone(), id, to_insert.clone()).unwrap();

            let data = db.find_by_id::<Value>(table_name.clone(), id).unwrap();
            assert_eq!(data, to_insert);

            let another_id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
            let another_to_insert: Value = json!({"id": another_id, "value": "another value"});
            db.insert_or_update::<Value>(table_name.clone(), another_id, another_to_insert.clone()).unwrap();

            let another_data = db.find_by_id::<Value>(table_name.clone(), another_id).unwrap(); 
            assert_eq!(another_data, another_to_insert)
        });
    }

    #[test]
    fn test_update() {
        run_with_file_create_teardown(|| {
            let mut db = Db::init(String::from(FILE_NAME)).unwrap();
            let table_name = String::from("sample");
            db.add_table(table_name.clone(), true).unwrap();

            let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
            let to_insert: Value = json!({"id": id, "value": "sample"});
            db.insert_or_update::<Value>(table_name.clone(), id, to_insert.clone()).unwrap();

            let data = db.find_by_id::<Value>(table_name.clone(), id).unwrap();
            assert_eq!(data, to_insert);

            let to_update: Value = json!({"id": id, "value": "updated"});
            db.insert_or_update::<Value>(table_name.clone(), id, to_update.clone()).unwrap();

            let updated_data = db.find_by_id::<Value>(table_name.clone(), id).unwrap(); 
            assert_eq!(updated_data, to_update)
        });
    }

    #[test]
    fn test_delete() {
        run_with_file_create_teardown(|| {
            let mut db = Db::init(String::from(FILE_NAME)).unwrap();
            let table_name = String::from("sample");
            db.add_table(table_name.clone(), true).unwrap();

            let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
            let to_insert: Value = json!({"id": id, "value": "sample"});
            db.insert_or_update::<Value>(table_name.clone(), id, to_insert.clone()).unwrap();

            let data = db.find_by_id::<Value>(table_name.clone(), id).unwrap();
            assert_eq!(data, to_insert);

            db.delete_by_id(table_name.clone(), id).unwrap();

            let data = db.find_by_id::<Value>(table_name.clone(), id);
            assert_eq!(data.is_none(), true); 
        });
    }

    #[test]
    fn test_delete_all() {
        run_with_file_create_teardown(|| {
            let mut db = Db::init(String::from(FILE_NAME)).unwrap();
            let table_name = String::from("sample");
            db.add_table(table_name.clone(), true).unwrap();

            let id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
            let to_insert: Value = json!({"id": id, "value": "sample"});
            db.insert_or_update::<Value>(table_name.clone(), id, to_insert.clone()).unwrap();

            let data = db.find_by_id::<Value>(table_name.clone(), id).unwrap();
            assert_eq!(data, to_insert);

            let another_id = db.get_increment_last_id(table_name.clone()).unwrap().unwrap();
            let another_to_insert: Value = json!({"id": another_id, "value": "another value"});
            db.insert_or_update::<Value>(table_name.clone(), another_id, another_to_insert.clone()).unwrap();

            let another_data = db.find_by_id::<Value>(table_name.clone(), another_id).unwrap(); 
            assert_eq!(another_data, another_to_insert);

            db.delete_all(table_name.clone()).unwrap();

            let all_result = db.find_all::<Value>(table_name.clone()).unwrap();

            assert_eq!(all_result.len(), 0);

        });
    }
 }
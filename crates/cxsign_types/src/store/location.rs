use crate::location::Location;
use cxsign_store::{AliasTable, DataBase, DataBaseTableTrait};
use log::warn;
use std::collections::HashMap;

pub struct LocationTable<'a> {
    db: &'a DataBase,
}

impl<'a> LocationTable<'a> {
    pub fn has_location(&self, location_id: i64) -> bool {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT count(*) FROM {} WHERE lid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        query.next().unwrap();
        query.read::<i64, _>(0).unwrap() > 0
    }
    pub fn add_location_or<O: Fn(&Self, i64, i64, &Location)>(
        &self,
        location_id: i64,
        course_id: i64,
        location: &Location,
        or: O,
    ) {
        let addr = location.get_addr();
        let lat = location.get_lat();
        let lon = location.get_lon();
        let alt = location.get_alt();
        let mut query =self.db.prepare(format!("INSERT INTO {}(lid,courseid,addr,lat,lon,alt) values(:lid,:courseid,:addr,:lat,:lon,:alt);",Self::TABLE_NAME)).unwrap();
        query
            .bind::<&[(_, sqlite::Value)]>(
                &[
                    (":lid", location_id.into()),
                    (":courseid", course_id.into()),
                    (":addr", addr.into()),
                    (":lat", lat.into()),
                    (":lon", lon.into()),
                    (":alt", alt.into()),
                ][..],
            )
            .unwrap();
        match query.next() {
            Ok(_) => (),
            Err(_) => or(self, location_id, course_id, location),
        }
    }
    pub fn delete_location(&self, location_id: i64) {
        self.db
            .execute(format!(
                "DELETE FROM {} WHERE lid={location_id};",
                Self::TABLE_NAME
            ))
            .unwrap();
        let alias_table = AliasTable::from_ref(self.db);
        let aliases = alias_table.get_aliases(location_id);
        for alias in aliases {
            alias_table.delete_alias(&alias)
        }
    }
    pub fn get_locations(&self) -> HashMap<i64, (i64, Location)> {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {};", Self::TABLE_NAME))
            .unwrap();
        let mut location_map = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let os_id = row.read("lid");
                let addr = row.read("addr");
                let lat = row.read("lat");
                let lon = row.read("lon");
                let alt = row.read("alt");
                let course_id = row.read("courseid");
                location_map.insert(os_id, (course_id, Location::new(addr, lon, lat, alt)));
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        location_map
    }
    pub fn get_location(&self, location_id: i64) -> (i64, Location) {
        let mut query = self
            .db
            .prepare(format!("SELECT * FROM {} WHERE lid=?;", Self::TABLE_NAME))
            .unwrap();
        query.bind((1, location_id)).unwrap();
        let c: Vec<sqlite::Row> = query
            .iter()
            .filter_map(|e| if let Ok(e) = e { Some(e) } else { None })
            .collect();
        let row = &c[0];
        let addr = row.read("addr");
        let lat = row.read("lat");
        let lon = row.read("lon");
        let alt = row.read("alt");
        let course_id = row.read("courseid");
        (course_id, Location::new(addr, lon, lat, alt))
    }
    pub fn get_location_by_alias(&self, alias: &str) -> Option<Location> {
        AliasTable::from_ref(self.db)
            .get_location_id(alias)
            .map(|id| self.get_location(id).1)
    }
    pub fn get_location_map_by_course(&self, course_id: i64) -> HashMap<i64, Location> {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT * FROM {} WHERE courseid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, course_id)).unwrap();
        let mut location_map = HashMap::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let location_id = row.read("lid");
                let addr = row.read("addr");
                let lon = row.read("lon");
                let lat = row.read("lat");
                let alt = row.read("alt");
                location_map.insert(location_id, Location::new(addr, lon, lat, alt));
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        location_map
    }
    pub fn get_location_list_by_course(&self, course_id: i64) -> Vec<Location> {
        let mut query = self
            .db
            .prepare(format!(
                "SELECT * FROM {} WHERE courseid=?;",
                Self::TABLE_NAME
            ))
            .unwrap();
        query.bind((1, course_id)).unwrap();
        let mut location_list = Vec::new();
        for c in query.iter() {
            if let Ok(row) = c {
                let addr = row.read("addr");
                let lat = row.read("lat");
                let lon = row.read("lon");
                let alt = row.read("alt");
                location_list.push(Location::new(addr, lon, lat, alt));
            } else {
                warn!("位置解析行出错：{c:?}.");
            }
        }
        location_list
    }
}

impl<'a> DataBaseTableTrait<'a> for LocationTable<'a> {
    const TABLE_ARGS: &'static str = "lid INTEGER UNIQUE NOT NULL,courseid INTEGER NOT NULL,addr TEXT NOT NULL,lon TEXT NOT NULL,lat TEXT NOT NULL,alt TEXT NOT NULL";
    const TABLE_NAME: &'static str = "location";

    fn from_ref(db: &'a DataBase) -> Self {
        Self { db }
    }
}

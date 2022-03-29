use crate::prelude::*;

impl Database {
    pub fn save_info(pos: Vector2, scale: Vector2, name: &String) {
        let db = Self::get();

        let sql = format!("
        INSERT INTO ui_elements (
            name,
            pos_x, pos_y,
            scale_x, scale_y
        ) VALUES (
            '{name}',
            {}, {},
            {}, {}
        )",
        pos.x, pos.y,
        scale.x, scale.y);
        
        let mut s = db.prepare(&sql).unwrap();

        // error is entry already exists
        if let Err(_) = s.execute([]) {
            // trance!("updating diff: {diff}");
            let sql = format!(
                "UPDATE ui_elements SET pos_x={}, pos_y={}, scale_x={}, scale_y={} WHERE name='{name}'", 
                pos.x, pos.y,
                scale.x, scale.y
            );
            let mut s = db.prepare(&sql).unwrap();

            if let Err(e) = s.execute([]) {
                error!("Error inserting/updateing ui_elements table: {e}")
            }
        }
    }

    pub fn get_info(name: &String) -> Option<(Vector2, Vector2)> {
        let sql = format!("SELECT pos_x, pos_y, scale_x, scale_y FROM ui_elements WHERE name='{name}'");

        let db = Self::get();
        let mut s = db.prepare(&sql).unwrap();
        let res = s.query_map([], |row| Ok((
            Vector2::new(
                row.get::<&str, f64>("pos_x")?,
                row.get::<&str, f64>("pos_y")?,
            ), Vector2::new(
                row.get::<&str, f64>("scale_x")?,
                row.get::<&str, f64>("scale_y")?,
            )
        )));

        if let Ok(mut rows) = res {
            rows.find_map(|r|r.ok())
        } else {
            None
        }
    }
}
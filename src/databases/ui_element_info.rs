use crate::prelude::*;

impl Database {
    pub async fn save_element_info(pos: Vector2, scale: Vector2, visible: bool, name: &String) {
        let db = Self::get().await;

        let sql = format!("
        INSERT INTO ui_elements (
            name, visible,
            pos_x, pos_y,
            scale_x, scale_y
        ) VALUES (
            '{name}', {visible},
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
                "UPDATE ui_elements SET pos_x={}, pos_y={}, scale_x={}, scale_y={}, visible={visible} WHERE name='{name}'", 
                pos.x, pos.y,
                scale.x, scale.y
            );
            let mut s = db.prepare(&sql).unwrap();

            if let Err(e) = s.execute([]) {
                error!("Error inserting/updateing ui_elements table: {e}")
            }
        }
    }

    pub async fn get_element_info(name: &String) -> Option<(Vector2, Vector2, bool)> {
        let sql = format!("SELECT pos_x, pos_y, scale_x, scale_y, visible FROM ui_elements WHERE name='{name}'");

        let db = Self::get().await;
        let mut s = db.prepare(&sql).unwrap();
        let res = s.query_map([], |row| Ok((
            Vector2::new(
                row.get::<&str, f64>("pos_x")?,
                row.get::<&str, f64>("pos_y")?,
            ), Vector2::new(
                row.get::<&str, f64>("scale_x")?,
                row.get::<&str, f64>("scale_y")?,
            ),
            row.get::<&str, bool>("visible")?,
        )));

        if let Ok(mut rows) = res {
            rows.find_map(|r|r.ok())
        } else {
            None
        }
    }

    
    pub async fn clear_element_info(name: &String) {
        let db = Self::get().await;
        let sql = format!("DELETE FROM ui_elements WHERE name='{name}'");
        
        let mut s = db.prepare(&sql).unwrap();
        if let Err(e) = s.execute([]) {
            error!("Error inserting/updateing ui_elements table: {e}")
        }
    }

}
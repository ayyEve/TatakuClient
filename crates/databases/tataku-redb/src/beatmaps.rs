use tataku_engine::*;

use engine::data;
use crate::wrapper::Wrapper;

use redb::{
    TableDefinition,
    
    ReadableTable as _,
    ReadableDatabase as _,
};


// beatmaps
const BEATMAPS:TableDefinition<String, Wrapper<BeatmapMeta>> = TableDefinition::new("beatmaps");
const IGNORE_BEATMAPS:TableDefinition<String, Wrapper<data::IgnoredBeatmap>> = TableDefinition::new("ignored_beatmaps");
const BEATMAP_COLLECTIONS:TableDefinition<String, Wrapper<data::BeatmapCollection>> = TableDefinition::new("beatmap_collections");

impl engine::database::BeatmapProvider for crate::Database {
    fn get_beatmaps(&self) -> tataku::Result<Vec<Arc<engine::BeatmapMeta>>> {
        let read = self.0
            .begin_read()
            .map_err(Self::transaction_err)?;

        let table = read
            .open_table(BEATMAPS)
            .map_err(Self::table_err)?;

        let list = table
            .iter()
            .map_err(Self::storage_err)?
            .filter_map(Result::ok)
            .map(|(_, i)| Arc::new(i.value()))
            .collect::<Vec<_>>();

        Ok(list)
    }

    fn add_beatmaps(&mut self, maps: &[Arc<engine::BeatmapMeta>]) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(BEATMAPS)
            .map_err(Self::table_err)?;

        for i in maps.iter() {
            let key = i.beatmap_hash.to_string();
            
            table.insert(key, &**i).map_err(Self::storage_err)?;
        }

        Ok(())
    }

    fn clear_all_beatmaps(&mut self) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(BEATMAPS)
            .map_err(Self::table_err)?;
        
        table.retain(|_,_| false).map_err(Self::storage_err)?;

        Ok(())
    }


    // ignored beatmaps
    fn get_ignored_beatmaps(&self) -> tataku::Result<Vec<data::IgnoredBeatmap>> {
        let read = self.0
            .begin_read()
            .map_err(Self::transaction_err)?;

        let table = read
            .open_table(IGNORE_BEATMAPS)
            .map_err(Self::table_err)?;

        let list = table
            .iter()
            .map_err(Self::storage_err)?
            .filter_map(Result::ok)
            .map(|(_, i)| i.value())
            .collect::<Vec<_>>();

        Ok(list)
    }

    fn add_ignored_beatmap(&mut self, ignored: &data::IgnoredBeatmap) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(IGNORE_BEATMAPS)
            .map_err(Self::table_err)?;

        let key = ignored.as_str().into_owned();
        table.insert(key, ignored).map_err(Self::storage_err)?;

        Ok(())
    }

    fn remove_ignored_beatmap(&mut self, ignored: &data::IgnoredBeatmap) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(IGNORE_BEATMAPS)
            .map_err(Self::table_err)?;

        let key = ignored.as_str().into_owned();
        table.remove(key).map_err(Self::storage_err)?;

        Ok(())
    }


    // beatmap collections
    fn get_beatmap_collections(&self) -> tataku::Result<Vec<data::BeatmapCollection>> {
        let read = self.0
            .begin_read()
            .map_err(Self::transaction_err)?;

        let table = read
            .open_table(BEATMAP_COLLECTIONS)
            .map_err(Self::table_err)?;

        let list = table
            .iter()
            .map_err(Self::storage_err)?
            .filter_map(Result::ok)
            .map(|(_, i)| i.value())
            .collect::<Vec<_>>();

        Ok(list)
    }

    fn update_beatmap_collection(&mut self, collection: &data::BeatmapCollection) -> tataku::Result<()> {
        self.add_beatmap_collection(collection)
    }

    fn add_beatmap_collection(&mut self, collection: &data::BeatmapCollection) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(BEATMAP_COLLECTIONS)
            .map_err(Self::table_err)?;

        let key = collection.name.clone();
        table.insert(key, collection).map_err(Self::storage_err)?;

        Ok(())
    }

    fn remove_beatmap_collection(&mut self, name: &str) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(BEATMAP_COLLECTIONS)
            .map_err(Self::table_err)?;

        let key = name.to_owned();
        table.remove(key).map_err(Self::storage_err)?;

        Ok(())
    }
}

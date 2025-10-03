use tataku_engine::*;
use crate::wrapper::Wrapper;

use redb::{
    TableDefinition,
    
    ReadableTable as _,
    ReadableDatabase as _,
};

const SCORES:TableDefinition<String, Wrapper<common::Score>> = TableDefinition::new("scores");

impl engine::database::ScoreProvider for crate::Database {
    fn get_scores(
        &self, 
        map: common::Md5Hash, 
        playmode: &str,
        _infos: &tataku_engine::gameplay::GamemodeInfos,
    ) -> tataku::Result<Vec<common::Score>> {
        let read = self.0
            .begin_read()
            .map_err(Self::transaction_err)?;

        let table = read
            .open_table(SCORES)
            .map_err(Self::table_err)?;

        let list = table
            .iter()
            .map_err(Self::storage_err)?
            .filter_map(Result::ok)
            .map(|(_, i)| i.value())
            .filter(|s| s.beatmap_hash == map && s.playmode == playmode)
            .collect::<Vec<_>>();

        Ok(list)
    }

    fn add_score(&mut self, score: &common::Score) -> tataku::Result<()> {
        let write = self.0
            .begin_write()
            .map_err(Self::transaction_err)?;

        let mut table = write
            .open_table(SCORES)
            .map_err(Self::table_err)?;

        let key = score.hash();
        table.insert(key, score).map_err(Self::storage_err)?;

        Ok(())
    }
}

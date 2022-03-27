use crate::prelude::*;

pub struct JudgmentImageHelper {
    miss: Option<Image>,
    x50: Option<Image>,
    x100: Option<Image>,
    x300: Option<Image>,
    geki: Option<Image>,
    katu: Option<Image>,
}
impl JudgmentImageHelper {
    pub fn new(miss: Option<Image>, x50: Option<Image>, x100: Option<Image>, x300: Option<Image>, katu: Option<Image>, geki: Option<Image>) -> Self {
        Self {
            miss,
            x50,
            x100,
            x300,
            katu,
            geki,
        }
    }

    pub fn get_from_scorehit(&self, hit: &ScoreHit) -> Option<Image> {
        match hit {
            ScoreHit::Miss => self.miss.clone(),
            ScoreHit::X50 => self.x50.clone(),
            ScoreHit::X100 => self.x100.clone(),
            ScoreHit::X300 => self.x300.clone(),
            ScoreHit::Xgeki => self.geki.clone(),
            ScoreHit::Xkatu => self.katu.clone(),

            ScoreHit::None
            | ScoreHit::Other(_, _) => None,
        }
    }
}
use crate::prelude::*;

pub struct JudgmentImageHelper {
    images: HashMap<String, Option<Image>>,
    
}
impl JudgmentImageHelper {
    pub async fn new<J:HitJudgments>(judge: J) -> Self {
        let mut images = HashMap::new();

        for i in judge.variants().iter() {
            let k = i.as_str_internal().to_owned();
            let img = i.tex_name();
            if img.is_empty() { continue }

            images.insert(k, SkinManager::get_texture(img, true).await);
        }

        Self {
            images
        }
    }

    pub fn get_from_scorehit<J:HitJudgments>(&self, judge: &J) -> Option<Image> {
        self.images.get(judge.as_str_internal()).unwrap().clone()
    }
}
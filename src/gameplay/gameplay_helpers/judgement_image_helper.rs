use crate::prelude::*;

pub struct JudgmentImageHelper {
    images: HashMap<String, Option<Image>>,
    variants: Vec<Box<dyn HitJudgments>>
}
impl JudgmentImageHelper {
    pub async fn new<J:HitJudgments>(judge: J) -> Self {
        let mut images = HashMap::new();

        let variants = judge.variants();
        for i in variants.iter() {
            let k = i.as_str_internal().to_owned();
            let img = i.tex_name();
            if img.is_empty() { continue }

            images.insert(k, SkinManager::get_texture(img, true).await);
        }

        Self {
            images,
            variants
        }
    }

    pub fn get_from_scorehit<J:HitJudgments>(&self, judge: &J) -> Option<Image> {
        self.images.get(judge.as_str_internal()).unwrap().clone()
    }

    pub async fn reload_skin(&mut self) {
        self.images.clear();
        
        for i in self.variants.iter() {
            let k = i.as_str_internal().to_owned();
            let img = i.tex_name();
            if img.is_empty() { continue }

            self.images.insert(k, SkinManager::get_texture(img, true).await);
        }
    }
}
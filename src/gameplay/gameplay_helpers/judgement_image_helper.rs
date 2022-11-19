use crate::prelude::*;

pub struct JudgmentImageHelper {
    images: HashMap<String, Option<Image>>,
    variants: Vec<Box<dyn HitJudgments>>
}
impl JudgmentImageHelper {
    pub async fn new<J:HitJudgments>(judge: J) -> Self {
        let mut s = Self {
            images: HashMap::new(),
            variants: judge.variants()
        };

        s.reload_skin().await;
        s
    }

    pub fn get_from_scorehit<J:HitJudgments>(&self, judge: &J) -> Option<Image> {
        self.images.get(judge.as_str_internal()).cloned().flatten()
    }

    pub async fn reload_skin(&mut self) {
        self.images.clear();
        
        for i in self.variants.iter() {
            let k = i.as_str_internal().to_owned();
            let img = i.tex_name();
            if img.is_empty() { continue }
            let tex = SkinManager::get_texture(img, true).await;
            // println!("trying to load tex {img}, got? {}", tex.is_some());
            self.images.insert(k, tex);
        }
    }
}
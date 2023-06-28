use crate::prelude::*;

pub struct JudgmentImageHelper {
    images: HashMap<String, Option<Animation>>,
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

    pub fn get_from_scorehit<J:HitJudgments>(&self, judge: &J) -> Option<Animation> {
        self.images.get(judge.as_str_internal()).cloned().flatten()
    }

    pub async fn reload_skin(&mut self) {
        self.images.clear();
        let skin = CurrentSkinHelper::new();
        
        for i in self.variants.iter() {
            let k = i.as_str_internal().to_owned();
            let img = i.tex_name();
            if img.is_empty() { continue }

            // try to load an animation
            let mut textures = Vec::new();
            loop {
                let img = img.to_owned() + "-" + &textures.len().to_string();
                if let Some(tex) = SkinManager::get_texture(img, true).await {
                    textures.push(tex);
                } else {
                    break;
                }
            }

            // if there was no animation, try loading a static image (no -num)
            if textures.is_empty() {
                if let Some(tex) = SkinManager::get_texture(img, true).await {
                    textures.push(tex);
                }
            }

            // debug!("trying to load tex {img}, got? {}", tex.is_some());
            if textures.is_empty() {
                self.images.insert(k, None);
            } else {
                let size = textures[0].size();
                let base_scale = textures[0].base_scale;
                let frametime = 1000.0 / skin.animation_framerate as f32;
                let (frames, delays) = textures.into_iter().map(|t|(t.tex, frametime)).unzip();

                let animation = Animation::new(Vector2::ZERO, size, frames, delays, base_scale);
                self.images.insert(k, Some(animation));
            }

        }
    }
}

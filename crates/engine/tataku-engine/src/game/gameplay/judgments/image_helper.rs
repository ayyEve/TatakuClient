use crate::*;
use tataku::Vector2;
use gameplay::judgments::HitJudgment;

pub struct JudgmentImageHelper {
    images: HashMap<String, Option<graphics::Animation>>,
    variants: Vec<HitJudgment>
}
impl JudgmentImageHelper {
    pub fn new(variants: Vec<HitJudgment>) -> Self {
        Self {
            images: HashMap::new(),
            variants
        }
    }

    pub fn get_from_scorehit(&self, judge: &HitJudgment) -> Option<graphics::Animation> {
        self.images.get(judge.id).cloned().flatten()
    }

    #[cfg(feature="graphics")]
    pub fn reload_skin(&mut self, skin_manager: &mut dyn graphics::SkinProvider) {
        self.images.clear();

        for i in self.variants.iter() {
            let k = i.id.to_owned();
            if i.tex_name.is_empty() { continue }

            // try to load an animation
            let mut textures = Vec::new();
            let img = i.tex_name;
            loop {
                let img = img.to_owned() + "-" + &textures.len().to_string();
                if let Some(tex) = skin_manager.get_texture(
                    Path::new(&img), 
                    &graphics::TextureSource::Skin, 
                    graphics::SkinUsage::Game, 
                    false
                ) {
                    textures.push(tex);
                } else {
                    break;
                }
            }

            // if there was no animation, try loading a static image (no -num)
            if textures.is_empty() && let Some(tex) = skin_manager.get_texture(
                Path::new(&img), 
                &graphics::TextureSource::Skin, 
                graphics::SkinUsage::Game, 
                false
            ) {
                textures.push(tex);
            }

            // debug!("trying to load tex {img}, got? {}", tex.is_some());
            if textures.is_empty() {
                self.images.insert(k, None);
            } else {
                let size = textures[0].size();
                let base_scale = textures[0].base_scale;
                let frametime = 1000.0 / skin_manager.skin().animation_framerate as f32;
                let frames = textures
                    .into_iter()
                    .map(|t| t.tex)
                    .collect();

                let animation = graphics::Animation::new(
                    Vector2::ZERO, 
                    size, 
                    frames, 
                    frametime, 
                    base_scale
                );
                self.images.insert(k, Some(animation));
            }

        }
    }
}

use crate::*;
use std::sync::Arc;
use std::ops::Range;
use rand::{ rngs::ThreadRng, Rng };

pub struct Emitter {
    /// time of last spawned particle
    last_time: f32,

    /// long to wait before spawning a particle (ms)
    pub spawn_delay: f32,

    /// override if this should emit particles or not
    pub should_emit: bool,

    /// position of the emitter
    pub position: Vector2,

    /// how long the particle will live (ms)
    life: EmitterVal,
    angle: EmitterVal,
    speed: EmitterVal,
    scale: EmitterVal,
    opacity: EmitterVal,
    rotation: EmitterVal,

    pub color: Color,

    pub image: Arc<TextureReference>,
    
    pool: Arc<RwLock<Pool<Particle>>>,
}
impl Emitter {
    pub fn new(
        time: f32,

        spawn_delay: f32, 
        position: Vector2, 
        angle: EmitterVal,
        speed: EmitterVal,
        scale: EmitterVal,
        life: Range<f32>,
    
        opacity: EmitterVal,
        rotation: EmitterVal,
    
        color: Color,
        image: Arc<TextureReference>,
    ) -> Self {
        let capacity = (life.end * spawn_delay) as usize;

        let particle = Particle {
            image: *image,
            ..Default::default()
        };
        let pool = Arc::new(RwLock::new(
            Pool::new_cloning(capacity, particle)
        ));
        
        Self {
            should_emit: true,
            spawn_delay,
            position,
            angle,
            speed,
            scale,
            life: EmitterVal::init_only(life),
            opacity,
            rotation,
            color,
            image,
            pool,
            last_time: time,
        }
    }

    pub fn update(&mut self, time: f32) {
        if self.last_time + self.spawn_delay < time {
            self.last_time = time;
            if !self.should_emit || self.image.is_empty() { return }
            
            let mut rng = rand::rng();

            let mut lock = self.pool.write();
            if let Some(particle) = lock.next() {
                particle.position = self.position;

                let angle = init_val(&self.angle, &mut rng);
                let speed = init_val(&self.speed, &mut rng);
                particle.velocity = Vector2::from_angle(angle) * speed;

                particle.scale = init_val(&self.scale, &mut rng);
                particle.rotation = init_val(&self.rotation, &mut rng);
                particle.lifetime = init_val(&self.life, &mut rng);
                particle.lifetime_max = particle.lifetime;

                let opacity = init_val(&self.opacity, &mut rng);
                particle.color = self.color.alpha(opacity);
                particle.image = *self.image;
            }

        }
    }

    // TODO: can we move this into a shader?
    // all the positions etc are generated on the gpu so its a bit silly to do this
    #[cfg(feature="graphics")]
    pub fn draw(&self, options: DrawOptions, list: &mut RenderableCollection) {
        let lock = self.pool.read();

        for i in lock.iter_used() {
            let mut image = Image::new(Arc::new(i.image), 1.0);
            image.color = i.color;

            let transform= Transform {
                origin: image.size() / 2.0,
                scale: Vector2::ONE * i.scale,
                ..Transform::identity()
            };

            list.push(image.with_transform(transform.matrix()).merge_draw_options(options));
        }
    }

    /// sets all used particles to unused
    pub fn reset(&mut self, time: f32) {
        self.pool.write().clear();
        self.last_time = time;
    }

    pub fn get_ref(&self) -> EmitterReference {
        EmitterReference {
            info: EmitterInfo::new(&self.scale, &self.opacity, &self.rotation),
            pool: Arc::downgrade(&self.pool)
        }
    }
}


#[derive(Clone, Default2)]
#[derive(ChainableInitializer)]
pub struct EmitterBuilder {
    #[chain] spawn_delay: f32,
    #[chain] position: Vector2,
    #[chain] life: Range<f32>,
    #[chain] angle: EmitterVal,
    #[chain] speed: EmitterVal,
    #[chain] scale: EmitterVal,
    #[chain] opacity: EmitterVal,
    #[chain] rotation: EmitterVal,
    #[chain] color: Color,
    #[chain] image: Arc<TextureReference>,
    #[default(true)]
    #[chain] should_emit: bool,
}
impl EmitterBuilder {
    pub fn build(self, time: f32) -> Emitter {
        let mut e = Emitter::new(
            time,
            self.spawn_delay,
            self.position,
            self.angle,
            self.speed,
            self.scale,
            self.life,
            self.opacity,
            self.rotation,
            self.color,
            self.image,
        );
        e.should_emit = self.should_emit;
        e
    }
}

fn init_val(range: &EmitterVal, rng: &mut ThreadRng) -> f32 {
    if range.initial.start == range.initial.end {
        range.initial.end
    } else {
        rng.random_range(range.initial.clone())
    }
}

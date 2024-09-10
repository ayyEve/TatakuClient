use rand::{rngs::ThreadRng, Rng};

use crate::prelude::*;

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
    pub blend_mode: BlendMode,
    
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
        blend_mode: BlendMode,
    ) -> Self {
        let capacity = (life.end * spawn_delay) as usize;

        let mut particle = Particle::default();
        particle.image = *image;
        let pool = Arc::new(RwLock::new(Pool::new_cloning(capacity, particle)));
        

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
            blend_mode,
        }
    }

    /// helper for generating a random value from the init range in `range`
    fn init_val(range: &EmitterVal, rng: &mut ThreadRng) -> f32 {
        if range.initial.start == range.initial.end {
            range.initial.end
        } else {
            rng.gen_range(range.initial.clone())
        }
    }

    pub fn update(&mut self, time: f32) {
        if self.last_time + self.spawn_delay < time {
            self.last_time = time;
            if !self.should_emit || self.image.is_empty() { return }

            let mut rng = rand::thread_rng();

            let mut lock = self.pool.write();
            if let Some(particle) = lock.next() {
                particle.position = self.position;

                let angle = Self::init_val(&self.angle, &mut rng);
                let speed = Self::init_val(&self.speed, &mut rng);
                particle.velocity = Vector2::from_angle(angle) * speed;

                particle.scale = Self::init_val(&self.scale, &mut rng);
                particle.rotation = Self::init_val(&self.rotation, &mut rng);
                particle.lifetime = Self::init_val(&self.life, &mut rng);
                particle.lifetime_max = particle.lifetime;

                let opacity = Self::init_val(&self.opacity, &mut rng);
                particle.color = self.color.alpha(opacity);
                particle.image = *self.image;
            }

        }
    }

    pub fn draw(&self, list: &mut RenderableCollection) {
        let lock = self.pool.read();

        for i in lock.iter_used() {
            let mut image = Image::new(i.position, Arc::new(i.image), Vector2::ONE);
            image.color = i.color;
            image.scale = Vector2::ONE * i.scale;
            image.set_blend_mode(self.blend_mode);

            list.push(image);
        }
    }

    /// sets all used particles to unused
    pub fn reset(&mut self, time: f32) {
        self.pool.write().clear();
        self.last_time = time;
    }

    //     // TODO: !!!!!!!!!!!!!!
    // pub fn get_ref(&self) -> Box<dyn EmitterReference> {
    //     let info = EmitterInfo::new(&self.scale, &self.opacity, &self.rotation);
    //     let a = Arc::downgrade(&self.pool) //, info);
    //     Box::new(EmitterRef(a, info))
    // }
}

// struct EmitterRef(Weak<RwLock<Pool<Particle>>>, EmitterInfo);
// impl EmitterReference for EmitterRef {
//     fn get_info(&self) -> EmitterInfo {
//         self.1.clone()
//     }

//     fn get_pool(&self) -> Option<Self::T<'_>> {
//         let a = self.0.upgrade()?;
//         a.write()
//     }
// }
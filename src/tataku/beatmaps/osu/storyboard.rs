

use ayyeve_piston_ui::render::Vector2;

#[allow(unused, dead_code)]
pub struct StoryboardSpriteDef {
    /**
     * the layer the object appears on
     */
    pub layer: Layer,

    /**
     * Where on the image should osu! consider that image's origin (coordinate) to be. 
     * This affects the (x) and (y) values, as well as several other command-specific behaviors. 
     * For example, choosing (origin) = TopLeft will let the (x),(y) values determine, where the top left corner of the image itself should be on the screen.
     */
    pub origin: Origin,

    /**
     * https://osu.ppy.sh/wiki/en/Storyboard/Scripting/Objects
     */
    pub filepath: String,

    /**
     * (x) and (y) are the x-/y-coordinates of where the object should be, by default respectively. The interpretation of this depends on the value of (origin); 
     * for instance, to place a 640x480 image as your background, the values could be: 
     * origin = TopLeft, x = 0, y = 0 
     * origin = Centre, x = 320, y = 240 
     * origin = BottomRight, x = 640, y = 480 
     */
    pub pos: Vector2
}

#[allow(unused, dead_code)]
pub struct StoryboardAnimationDef {
    /**
     * the layer the object appears on
     */
    pub layer: Layer,

    /**
     * Where on the image should osu! consider that image's origin (coordinate) to be. 
     * This affects the (x) and (y) values, as well as several other command-specific behaviors. 
     * For example, choosing (origin) = TopLeft will let the (x),(y) values determine, where the top left corner of the image itself should be on the screen.
     */
    pub origin: Origin,

    /**
     * https://osu.ppy.sh/wiki/en/Storyboard/Scripting/Objects
     */
    pub filepath: String,

    /**
     * (x) and (y) are the x-/y-coordinates of where the object should be, by default respectively. The interpretation of this depends on the value of (origin); 
     * for instance, to place a 640x480 image as your background, the values could be: 
     * origin = TopLeft, x = 0, y = 0 
     * origin = Centre, x = 320, y = 240 
     * origin = BottomRight, x = 640, y = 480 
     */
    pub pos: Vector2,

    /**
     * indicates how many frames the animation has. If we have "sample0.png" and "sample1.png"
    */
    pub frame_count: u16,

    /**
     * indicates how many milliseconds should be in between each frame. For instance, if we wanted our animation to advance at 2 frames per second, frameDelay = 500.
     */
    pub frame_delay: f32,

    /**
     * indicates if the animation should loop or not.
     */
    pub loop_type: LoopType,
}

#[allow(unused, dead_code)]
pub enum StoryboardElementDef {
    Sprite(StoryboardSpriteDef),
    Animation(StoryboardAnimationDef)
}

#[allow(unused, dead_code)]
pub struct StoryboardElement {
    pub def: StoryboardElementDef
}


#[allow(unused, dead_code)]
pub enum StoryboardTransform {

}

#[allow(unused, dead_code)]
pub enum Origin {
    TopLeft = 0,
    Centre = 1,
    CentreLeft = 2,
    TopRight = 3,
    BottomCentre = 4,
    TopCentre = 5,
    Custom = 6, // (same effect as TopLeft, but should not be used)
    CentreRight = 7,
    BottomLeft = 8,
    BottomRight = 9
}

#[allow(unused, dead_code)]
pub enum LoopType {
    LoopForever = 0,
    LoopOnce = 1
}

#[allow(unused, dead_code)]
pub enum Layer {
    Background = 0,
    Fail = 1,
    Pass = 2,
    Foreground = 3
}
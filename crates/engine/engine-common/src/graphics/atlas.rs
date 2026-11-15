use guillotiere::*;

pub const ATLAS_PADDING:u32 = 2;

pub struct Atlas {
    width: u32,
    height: u32,

    allocators: Vec<AtlasAllocator>,
} 
impl Atlas {
    pub fn new(width: u32, height: u32, layers: u32) -> Self {
        let allocators =  (0..layers)
            .map(|_| AtlasAllocator::new(size2(width as i32, height as i32)))
            .collect();
        
        Self {
            width,
            height,
            allocators,
        }
    }

    /// Add an empty layer to the atlas
    pub fn add_layer(&mut self) {
        self.allocators.push(AtlasAllocator::new(size2(
            self.width as i32, 
            self.height as i32
        )));
    }

    pub fn layer_count(&self) -> usize {
        self.allocators.len()
    }

    pub fn try_insert(&mut self, width: u32, height: u32) -> Option<AtlasData> {
        if width == 0 || height == 0 { return Some(TextureReference::empty()) }
        self.allocators
            .iter_mut()
            .enumerate()
            .find_map(|(n, alloc)| alloc.allocate(size2((width + ATLAS_PADDING * 2) as i32, (height + ATLAS_PADDING * 2) as i32)).map(|a|(n as u32, a)))
            .map(|(layer, i)| AtlasData::from_alloc(i, layer, self.width, self.height))
    }
    
    pub fn remove_entry(&mut self, entry: TextureReference) {
        if entry.is_empty() { return }
        self.allocators
            .get_mut(entry.layer as usize)
            .unwrap()
            .deallocate(AllocId::deserialize(entry.id));
    }
}

pub type TextureReference = AtlasData;
#[derive(Copy, Clone, Debug)]
pub struct AtlasData {
    id: u32,
    pub tag: AtlasTag,

    pub x: u32,
    pub y: u32,
    pub layer: u32,
    
    pub width: u32,
    pub height: u32,
    pub uvs: Uvs,
}
impl AtlasData {
    pub fn new(
        [x, y]: [u32; 2],
        [w, h]: [u32; 2],
        layer: u32,
        id: u32,
        tag: AtlasTag,
        total_size: [u32; 2],
    ) -> Self {
        Self {
            id,
            tag,

            x,
            y,
            layer,
            width: w,
            height: h,

            uvs: Uvs::new(
                x, y, w, h, 
                total_size[0], 
                total_size[1]
            ),
        }
    }

    fn from_alloc(
        alloc_info: Allocation, 
        layer: u32, 
        total_width: u32, 
        total_height: u32
    ) -> Self {
        let [x, y] = alloc_info.rectangle.min.to_array();
        let [x2, y2] = alloc_info.rectangle.max.to_array();
        
        let [x, y, x2, y2] = [
            x as u32 + ATLAS_PADDING, 
            y as u32 + ATLAS_PADDING, 
            x2 as u32 - ATLAS_PADDING, 
            y2 as u32 - ATLAS_PADDING
        ];

        let w = x2 - x;
        let h = y2 - y;

        Self::new(
            [x, y],
            [w, h],
            layer,
            alloc_info.id.serialize(),
            AtlasTag::NonEmpty, 
            [ total_width, total_height ],
        )
    }

    pub fn is_empty(&self) -> bool {
        self.tag == AtlasTag::Empty
        || self.width == 0 || self.height == 0
    }

    pub fn empty() -> Self {
        Self {
            id: 0,
            tag: AtlasTag::Empty,
            x: 0,
            y: 0,
            layer: 0,
            width: 0,
            height: 0,
            uvs: Uvs::new(0,0,0,0,1,1),
        }
    }

}
impl Default for AtlasData {
    fn default() -> Self {
        Self::empty()
    }
}


#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum AtlasTag {
    Empty,
    #[default]
    NonEmpty,
    Glyph,
    Custom(u8),
}

#[derive(Copy, Clone, Debug)]
pub struct Uvs {
    pub tl: [f32; 2],
    pub tr: [f32; 2],
    pub bl: [f32; 2],
    pub br: [f32; 2],
}
impl Uvs {
    pub fn new(
        x: u32, 
        y: u32, 
        w: u32, 
        h: u32, 
        total_w: u32, 
        total_h: u32,
    ) -> Self {
        let [x, y, w, h] = [x as f32, y as f32, w as f32, h as f32];

        Self {
            tl: [x, y],
            tr: [x + w, y],
            bl: [x, y + h],
            br: [x + w, y + h],
        }.div([total_w as f32, total_h as f32])
    }

    fn div(mut self, max_size: [f32; 2]) -> Self {
        self.tl = div(self.tl, max_size);
        self.tr = div(self.tr, max_size);
        self.bl = div(self.bl, max_size);
        self.br = div(self.br, max_size);
        self
    }
}

fn div(a: [f32; 2], b: [f32; 2]) -> [f32; 2] {
    [
        a[0] / b[0],
        a[1] / b[1]
    ]
}

use crate::prelude::*;

#[derive(Reflect)]
#[reflect(display = "debug")]
#[derive(Debug, Clone, Default)]
pub struct EnumValues {
    // static enums
    pub sort_by: Vec<SortBy>,
    pub group_by: Vec<GroupBy>,
    pub vsync: Vec<Vsync>,
    pub performance_mode: Vec<PerformanceMode>,
    pub score_methods: Vec<ScoreRetreivalMethod>,

    pub playmodes: HashMap<String, PlaymodeReflect>,


    // modifiable enums
    pub skins: Vec<String>,
    pub themes: Vec<SelectedTheme>,
    pub monitors: Vec<FullscreenMonitor>,
}
impl EnumValues {
    pub fn new(infos: &GamemodeInfos) -> Self {
        let playmodes = infos
            .by_num
            .iter()
            .map(|g| (g.id.to_string(), PlaymodeReflect::new(g)))
            .collect();

        Self {
            sort_by: SortBy::list(),
            group_by: GroupBy::list(),
            score_methods: ScoreRetreivalMethod::list(),
            vsync: Vsync::list(),
            performance_mode: PerformanceMode::list(),
            playmodes,

            skins: Vec::new(),
            themes: vec![ SelectedTheme::Tataku, SelectedTheme::Osu ],
            monitors: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlaymodeReflect {
    id: String,
    display: String,
}
impl PlaymodeReflect {
    fn new(info: &GamemodeInfo) -> Self {
        Self {
            id: info.id.to_owned(),
            display: info.display_name.to_owned(),
        }
    }
}
impl Reflect for PlaymodeReflect {
    fn impl_get<'s, 'v>(
        &'s self, 
        mut path: ReflectPath<'v>
    ) -> ReflectResult<'v, MaybeOwnedReflect<'s>> {
        if !path.has_next() {
            Ok(MaybeOwnedReflect::Borrowed(&self.id))
        } else {
            Err(ReflectError::EntryNotExist { 
                entry: path.next().unwrap().into() 
            })
        }
    }
    fn impl_get_mut<'s, 'v>(
        &'s mut self, 
        mut path: ReflectPath<'v>
    ) -> ReflectResult<'v, &'s mut dyn Reflect> {
        if !path.has_next() {
            Ok(&mut self.id)
        } else {
            Err(ReflectError::EntryNotExist { 
                entry: path.next().unwrap().into() 
            })
        }
    }
    
    fn impl_display<'v>(
        &self, 
        _path: ReflectPath<'v>, 
        _precision: Option<usize>
    ) -> ReflectResult<'v, String> {
        Ok(self.display.clone())
    }

    fn duplicate(&self) -> Option<Box<dyn Reflect>> {
        Some(Box::new(self.clone()))
    }
    
    fn impl_insert<'v>(
        &mut self, 
        _path: ReflectPath<'v>, 
        _value: Box<dyn Reflect>
    ) -> ReflectResult<'v, ()> {
        Err(ReflectError::ImmutableContainer)
    }
}

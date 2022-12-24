use crate::prelude::*;

pub struct MenuStatsInfo {
    pub display_name: String,
    pub graph_type: GraphType,
    pub data: Arc<Vec<MenuStatsEntry>>,

    graph: Box<dyn StatsGraph>
}
impl MenuStatsInfo {
    pub fn new(display_name: impl ToString, graph_type: GraphType, data: Vec<MenuStatsEntry>) -> Self {
        let data = Arc::new(data);

        let graph:Box<dyn StatsGraph> = match graph_type {
            GraphType::Pie => Box::new(PieGraph::new(data.clone())),
            GraphType::Bar => Box::new(BarGraph::new(data.clone())),
            GraphType::Scatter => Box::new(ScatterGraph::new(data.clone())),
        };

        Self {
            display_name: display_name.to_string(),
            graph_type,
            data,
            graph
        }
    }

    pub fn draw(&self, bounds: &Rectangle, depth: f64, list: &mut RenderableCollection) {
        let font = get_font();

        // display name should be at the top with some margin above and below
        let display_text = Text::new(Color::BLACK, depth, bounds.current_pos, 30, self.display_name.clone(), font.clone());
        // display_text.center_text(bounds);
        // display_text.current_pos.y = 0.0;
        list.push(display_text);
        
        // ~half the remaining vertical should be for listing the values 
        let mut current_pos = bounds.current_pos + Vector2::new(0.0, 30.0 + 5.0);
        for i in self.data.iter() {
            if i.show_in_list {
                let text = format!("{}: {}", i.name, format_float(i.get_value(), 2));
                list.push(Text::new(i.color, depth, current_pos, 20, text, font.clone()));
            }
            current_pos += Vector2::with_y(20.0 + 5.0);
        }

        // there should be some margin between the list and the graph
        current_pos += Vector2::with_y(20.0);

        // the remaining space should be used for the graph
        let mut size = bounds.size - current_pos.y();
        if size.x < size.y { size.y = size.x; } else { size.x = size.y; }

        let y = bounds.current_pos.y + bounds.size.y - size.y;

        self.graph.draw(&Rectangle::bounds_only(Vector2::new(current_pos.x, y), size), depth, list);
    }
}


// #[derive(Debug, Copy, Clone)]
#[allow(unused)]
pub enum GraphType {
    Pie,
    Bar,
    Scatter,
}

pub struct MenuStatsEntry {
    pub name: String,
    pub value: MenuStatsValue,
    pub color: Color,
    pub show_in_graph: bool,
    pub show_in_list: bool,
    /// what to do with lists of values
    pub concat_method: ConcatMethod
}
impl MenuStatsEntry {
    pub fn new_f32(name: impl ToString, value: f32, color:Color, show_in_graph: bool, show_in_list: bool) -> Self {
        Self {
            name: name.to_string(),
            value: MenuStatsValue::Single(value),
            color,
            show_in_graph,
            show_in_list,
            concat_method: ConcatMethod::Sum
        }
    }
    pub fn new_list(name: impl ToString, values: Vec<f32>, color:Color, show_in_graph: bool, show_in_list: bool, concat_method: ConcatMethod) -> Self {
        Self {
            name: name.to_string(),
            value: MenuStatsValue::List(values),
            color,
            show_in_graph,
            show_in_list,
            concat_method
        }
    }

    pub fn get_value(&self) -> f32 {
        match &self.value {
            MenuStatsValue::Single(v) => *v,
            MenuStatsValue::List(list) => match self.concat_method {
                ConcatMethod::Sum => list.iter().sum(),
                ConcatMethod::Mean => list.iter().sum::<f32>() / list.len() as f32,
                ConcatMethod::StandardDeviation => {
                    // let mut total = 0.0;
                    // let mut _total = 0.0;
                    let mut total_all = 0.0;
                    // let mut count = 0.0;
                    // let mut _count = 0.0;
            
                    for &i in list.iter() {
                        total_all += i;
            
                        // if i > 0.0 {
                        //     total += i;
                        //     count += 1.0;
                        // } else {
                        //     _total += i;
                        //     _count += 1.0;
                        // }
                    }
            
                    let mean = total_all / list.len() as f32;
                    let mut variance = 0.0;
                    for &i in list.iter() {
                        variance += (i - mean).powi(2);
                    }
                    
                    (variance / list.len() as f32).sqrt()
                },
            }
        }
    }
}

#[allow(unused)]
pub enum ConcatMethod {
    /// total the values
    Sum,
    /// get the mean average
    Mean,
    /// get the standard dev
    StandardDeviation
}

#[derive(Clone)]
pub enum MenuStatsValue {
    Single(f32),
    List(Vec<f32>)
}


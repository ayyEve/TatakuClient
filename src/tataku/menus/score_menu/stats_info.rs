use crate::prelude::*;

pub struct MenuStatsInfo {
    pub display_name: String,
    pub graph_type: GraphType,
    pub data: Arc<Vec<MenuStatsEntry>>,

    graph: StatsGraph
}
impl MenuStatsInfo {
    pub fn new(display_name: impl ToString, graph_type: GraphType, data: Vec<MenuStatsEntry>) -> Self {
        let data = Arc::new(data);

        let graph = match graph_type {
            GraphType::Pie => StatsGraph::Pie(Box::new(PieGraph::new(data.clone()))),
            GraphType::Bar => StatsGraph::Bar(Box::new(BarGraph::new(data.clone()))),
            GraphType::Scatter => StatsGraph::Scatter(Box::new(ScatterGraph::new(data.clone()))),
        };

        Self {
            display_name: display_name.to_string(),
            graph_type,
            data,
            graph
        }
    }

    pub fn view(&self) -> IcedElement {
        use crate::prelude::iced_elements::*;

        col!(
            // display name should be at the top (TODO: with some margin above and below )
            Text::new(self.display_name.clone()).size(30.0).color(Color::BLACK).width(Fill),

            // ~half the remaining vertical should be for listing the values 
            col!(
                self.data.iter().filter(|i|i.show_in_list).map(|i|{
                    let text = format!("{}: {}", i.name, format_float(i.get_value(), 2));
                    Text::new(text).size(20.0).color(i.color).width(Fill).into_element()
                }).collect(),

                width = Fill,
                spacing = 5.0
            ),

            // there should be some margin between the list and the graph
            Space::new(Fill, Fixed(20.0)),
            
            // the remaining space should be used for the graph
            self.graph.view().width(Fill).height(Fill);

            width = Fill,
            height = Fill
        )
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


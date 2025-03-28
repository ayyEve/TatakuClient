use crate::prelude::*;

pub struct StatsInfo {
    pub display_name: String,
    pub graph_type: GraphType,
    pub data: Arc<Vec<StatsEntry>>,
}
impl StatsInfo {
    pub fn new(display_name: impl ToString, graph_type: GraphType, data: Vec<StatsEntry>) -> Self {
        Self {
            display_name: display_name.to_string(),
            graph_type,
            data: Arc::new(data),
        }
    }
}


#[derive(Copy, Clone, Debug)]
pub enum GraphType {
    Pie,
    Bar,
    Scatter,
}

pub struct StatsEntry {
    pub name: String,
    pub value: StatsValue,
    pub color: Color,
    pub show_in_graph: bool,
    pub show_in_list: bool,
    
    /// what to do with lists of values
    pub concat_method: ConcatMethod
}
impl StatsEntry {
    pub fn new_f32(name: impl ToString, value: f32, color:Color, show_in_graph: bool, show_in_list: bool) -> Self {
        Self {
            name: name.to_string(),
            value: StatsValue::Single(value),
            color,
            show_in_graph,
            show_in_list,
            concat_method: ConcatMethod::Sum
        }
    }
    pub fn new_list(name: impl ToString, values: Vec<f32>, color:Color, show_in_graph: bool, show_in_list: bool, concat_method: ConcatMethod) -> Self {
        Self {
            name: name.to_string(),
            value: StatsValue::List(values),
            color,
            show_in_graph,
            show_in_list,
            concat_method
        }
    }

    pub fn get_value(&self) -> f32 {
        match &self.value {
            StatsValue::Single(v) => *v,
            StatsValue::List(list) => match self.concat_method {
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
pub enum StatsValue {
    Single(f32),
    List(Vec<f32>)
}
